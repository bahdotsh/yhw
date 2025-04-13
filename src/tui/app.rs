use std::path::PathBuf;
use std::time::Duration;
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::analyzer::{DependencyAnalyzer, AnalysisResult};
use crate::tui::event::{EventHandler, EventConfig, Event as AppEvent};
use crate::tui::ui;

/// Sort options for dependencies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOption {
    /// Sort by name
    Name,
    /// Sort by usage count
    UsageCount,
    /// Sort by importance
    Importance,
    /// Sort by dependency type
    Type,
    /// Sort by removability
    Removable,
}

impl SortOption {
    /// Get display name for sort option
    pub fn as_str(&self) -> &'static str {
        match self {
            SortOption::Name => "Name",
            SortOption::UsageCount => "Usage Count",
            SortOption::Importance => "Importance",
            SortOption::Type => "Type",
            SortOption::Removable => "Removable",
        }
    }
    
    /// Get the next sort option
    pub fn next(&self) -> Self {
        match self {
            SortOption::Name => SortOption::UsageCount,
            SortOption::UsageCount => SortOption::Importance,
            SortOption::Importance => SortOption::Type,
            SortOption::Type => SortOption::Removable,
            SortOption::Removable => SortOption::Name,
        }
    }
}

/// Filter options for dependencies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterOption {
    /// Show all dependencies
    All,
    /// Show only normal dependencies
    Normal,
    /// Show only development dependencies
    Dev,
    /// Show only build dependencies
    Build,
    /// Show only unused dependencies
    Unused,
    /// Show only removable dependencies
    Removable,
}

impl FilterOption {
    /// Get display name for filter option
    pub fn as_str(&self) -> &'static str {
        match self {
            FilterOption::All => "All",
            FilterOption::Normal => "Normal",
            FilterOption::Dev => "Dev",
            FilterOption::Build => "Build",
            FilterOption::Unused => "Unused",
            FilterOption::Removable => "Removable",
        }
    }
    
    /// Get the next filter option
    pub fn next(&self) -> Self {
        match self {
            FilterOption::All => FilterOption::Normal,
            FilterOption::Normal => FilterOption::Dev,
            FilterOption::Dev => FilterOption::Build,
            FilterOption::Build => FilterOption::Unused,
            FilterOption::Unused => FilterOption::Removable,
            FilterOption::Removable => FilterOption::All,
        }
    }
}

/// Application state
pub struct App {
    /// Path to the project directory
    project_path: PathBuf,
    /// Analysis results
    pub analysis: Option<AnalysisResult>,
    /// Flag to indicate if the app should quit
    pub should_quit: bool,
    /// Current tab (0: Overview, 1: Details, 2: Removable)
    pub current_tab: usize,
    /// Selected dependency index
    pub selected_dependency: usize,
    /// Current sort option
    pub sort_option: SortOption,
    /// Whether to sort in reverse order
    pub sort_reverse: bool,
    /// Current filter option
    pub filter_option: FilterOption,
    /// Filtered dependency name (if any)
    pub filter_dep: Option<String>,
    /// Search query
    pub search_query: String,
    /// Whether the user is currently searching
    pub is_searching: bool,
    /// Whether to show the help popup
    pub show_help: bool,
    /// The current view in detail screen
    pub detail_view: usize,
    /// Whether to enable dependency graph visualization
    pub enable_dependency_graph: bool,
    /// Counter for animations
    pub tick_count: usize,
}

impl App {
    /// Create a new app instance
    pub fn new(project_path: PathBuf, filter_dep: Option<String>) -> Self {
        Self {
            project_path,
            analysis: None,
            should_quit: false,
            current_tab: 0,
            selected_dependency: 0,
            sort_option: SortOption::Name,
            sort_reverse: false,
            filter_option: FilterOption::All,
            filter_dep,
            search_query: String::new(),
            is_searching: false,
            show_help: false,
            detail_view: 0,
            enable_dependency_graph: false,
            tick_count: 0,
        }
    }
    
    /// Handle keyboard input
    pub fn handle_key_event(&mut self, key_event: KeyEvent) {
        // If in search mode, handle search input
        if self.is_searching {
            match key_event.code {
                KeyCode::Esc => {
                    self.is_searching = false;
                    self.search_query.clear();
                }
                KeyCode::Enter => {
                    self.is_searching = false;
                    // Search query is now set, will be used for filtering
                }
                KeyCode::Backspace => {
                    self.search_query.pop();
                }
                KeyCode::Char(c) => {
                    self.search_query.push(c);
                }
                _ => {}
            }
            return;
        }
        
        // Regular key handling
        match (key_event.code, key_event.modifiers) {
            (KeyCode::Char('q'), _) | (KeyCode::Esc, _) => {
                self.should_quit = true;
            }
            (KeyCode::Char('?'), _) => {
                self.show_help = !self.show_help;
            }
            (KeyCode::Tab, _) => {
                // Cycle through tabs
                self.current_tab = (self.current_tab + 1) % 3; // 3 tabs: Overview, Details, Removable
                self.selected_dependency = 0; // Reset selection when changing tabs
            }
            (KeyCode::BackTab, _) => {
                // Cycle through tabs backwards
                self.current_tab = (self.current_tab + 2) % 3;
                self.selected_dependency = 0; // Reset selection when changing tabs
            }
            (KeyCode::Down, _) | (KeyCode::Char('j'), _) => {
                if let Some(_analysis) = &self.analysis {
                    let len = self.filtered_dependencies().len().max(1);
                    self.selected_dependency = (self.selected_dependency + 1) % len;
                }
            }
            (KeyCode::Up, _) | (KeyCode::Char('k'), _) => {
                if let Some(_analysis) = &self.analysis {
                    let len = self.filtered_dependencies().len().max(1);
                    self.selected_dependency = (self.selected_dependency + len - 1) % len;
                }
            }
            (KeyCode::Char('s'), _) => {
                // Toggle sorting
                self.sort_option = self.sort_option.next();
            }
            (KeyCode::Char('r'), _) => {
                // Toggle sort direction
                self.sort_reverse = !self.sort_reverse;
            }
            (KeyCode::Char('f'), _) => {
                // Toggle filtering
                self.filter_option = self.filter_option.next();
                self.selected_dependency = 0; // Reset selection when changing filter
            }
            (KeyCode::Char('/'), _) => {
                // Enter search mode
                self.is_searching = true;
                self.search_query.clear();
            }
            (KeyCode::Right, _) | (KeyCode::Char('l'), _) => {
                // In details view, cycle through detail panels
                if self.current_tab == 1 {
                    self.detail_view = (self.detail_view + 1) % 3; // 3 detail views
                }
            }
            (KeyCode::Left, _) | (KeyCode::Char('h'), _) => {
                // In details view, cycle through detail panels backwards
                if self.current_tab == 1 {
                    self.detail_view = (self.detail_view + 2) % 3;
                }
            }
            _ => {}
        }
    }
    
    /// Run the analysis
    pub fn run_analysis(&mut self) -> Result<()> {
        let analyzer = DependencyAnalyzer::new(&self.project_path);
        self.analysis = Some(analyzer.analyze()?);
        
        // If a filter is specified, select that dependency
        if let Some(filter) = &self.filter_dep {
            if let Some(analysis) = &self.analysis {
                for (i, dep) in analysis.dependencies.iter().enumerate() {
                    if dep.name == *filter {
                        self.selected_dependency = i;
                        break;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Get filtered and sorted dependencies
    pub fn filtered_dependencies(&self) -> Vec<usize> {
        if let Some(analysis) = &self.analysis {
            let mut indices: Vec<usize> = (0..analysis.dependencies.len()).collect();
            
            // Filter dependencies
            indices.retain(|&i| {
                let dep = &analysis.dependencies[i];
                
                // First check filter option
                let filter_match = match self.filter_option {
                    FilterOption::All => true,
                    FilterOption::Normal => dep.dependency_type == crate::manifest::cargo::DependencyType::Normal,
                    FilterOption::Dev => dep.dependency_type == crate::manifest::cargo::DependencyType::Development,
                    FilterOption::Build => dep.dependency_type == crate::manifest::cargo::DependencyType::Build,
                    FilterOption::Unused => {
                        !*analysis.metrics.is_used.get(&dep.name).unwrap_or(&true)
                    },
                    FilterOption::Removable => {
                        analysis.metrics.removable_dependencies.contains(&dep.name)
                    },
                };
                
                // Then check search filter
                let search_match = if !self.search_query.is_empty() {
                    dep.name.to_lowercase().contains(&self.search_query.to_lowercase())
                } else {
                    true
                };
                
                // Return true only if both filters match
                filter_match && search_match
            });
            
            // Sort dependencies
            indices.sort_by(|&a, &b| {
                let dep_a = &analysis.dependencies[a];
                let dep_b = &analysis.dependencies[b];
                
                let cmp = match self.sort_option {
                    SortOption::Name => dep_a.name.cmp(&dep_b.name),
                    SortOption::UsageCount => {
                        let count_a = analysis.metrics.usage_count.get(&dep_a.name).unwrap_or(&0);
                        let count_b = analysis.metrics.usage_count.get(&dep_b.name).unwrap_or(&0);
                        count_a.cmp(count_b)
                    },
                    SortOption::Importance => {
                        let score_a = analysis.metrics.importance_scores.get(&dep_a.name).unwrap_or(&0.0);
                        let score_b = analysis.metrics.importance_scores.get(&dep_b.name).unwrap_or(&0.0);
                        score_a.partial_cmp(score_b).unwrap_or(std::cmp::Ordering::Equal)
                    },
                    SortOption::Type => {
                        // Compare dependency types based on their variant order
                        let type_order = |dep_type: &crate::manifest::cargo::DependencyType| -> u8 {
                            match dep_type {
                                crate::manifest::cargo::DependencyType::Normal => 0,
                                crate::manifest::cargo::DependencyType::Development => 1,
                                crate::manifest::cargo::DependencyType::Build => 2,
                            }
                        };
                        
                        type_order(&dep_a.dependency_type).cmp(&type_order(&dep_b.dependency_type))
                    },
                    SortOption::Removable => {
                        let rem_a = analysis.metrics.removable_dependencies.contains(&dep_a.name);
                        let rem_b = analysis.metrics.removable_dependencies.contains(&dep_b.name);
                        rem_a.cmp(&rem_b)
                    },
                };
                
                if self.sort_reverse {
                    cmp.reverse()
                } else {
                    cmp
                }
            });
            
            indices
        } else {
            Vec::new()
        }
    }
    
    /// Get the actual index of the selected dependency
    pub fn actual_selected_index(&self) -> Option<usize> {
        let filtered = self.filtered_dependencies();
        filtered.get(self.selected_dependency).copied()
    }
}

/// Run the TUI application
pub fn run(project_path: PathBuf, filter_dep: Option<String>, enable_deps: bool) -> Result<()> {
    // Set up terminal
    terminal::enable_raw_mode()?;
    std::io::stdout().execute(EnterAlternateScreen)?;
    
    // Create terminal backend and terminal
    let backend = CrosstermBackend::new(std::io::stdout());
    let mut terminal = Terminal::new(backend)?;
    
    // Create app state
    let mut app = App::new(project_path, filter_dep);
    
    // Enable dependency graph visualization if requested
    app.enable_dependency_graph = enable_deps;
    
    // Run analysis
    app.run_analysis()?;
    
    // Create event handler
    let event_config = EventConfig {
        tick_rate: Duration::from_millis(100), // Faster ticks for smoother animations
    };
    let event_handler = EventHandler::new(event_config);
    
    // Main loop
    while !app.should_quit {
        // Draw UI
        terminal.draw(|frame| ui::draw(frame, &app))?;
        
        // Handle events
        match event_handler.next()? {
            AppEvent::Key(key_event) => app.handle_key_event(key_event),
            AppEvent::Tick => {
                // Increment tick counter for animations
                app.tick_count = app.tick_count.wrapping_add(1);
            }
        }
    }
    
    // Restore terminal
    terminal::disable_raw_mode()?;
    std::io::stdout().execute(LeaveAlternateScreen)?;
    
    Ok(())
} 