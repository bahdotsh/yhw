use std::path::PathBuf;
use std::time::Duration;
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::analyzer::{DependencyAnalyzer, AnalysisResult};
use crate::tui::event::{EventHandler, EventConfig};
use crate::tui::ui;

/// Main application state
pub struct App {
    /// Path to the project being analyzed
    pub project_path: PathBuf,
    /// Optional dependency filter
    pub filter_dep: Option<String>,
    /// Analysis results
    pub analysis: Option<AnalysisResult>,
    /// Currently selected dependency index
    pub selected_dependency: usize,
    /// Current tab index
    pub current_tab: usize,
    /// Whether the application should exit
    pub should_quit: bool,
    /// Whether to show help
    pub show_help: bool,
}

impl App {
    /// Create a new application
    pub fn new(project_path: PathBuf, filter_dep: Option<String>) -> Self {
        Self {
            project_path,
            filter_dep,
            analysis: None,
            selected_dependency: 0,
            current_tab: 0,
            should_quit: false,
            show_help: false,
        }
    }
    
    /// Handle keyboard input
    pub fn handle_key_event(&mut self, key_event: KeyEvent) {
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
            }
            (KeyCode::BackTab, _) => {
                // Cycle through tabs backwards
                self.current_tab = (self.current_tab + 2) % 3;
            }
            (KeyCode::Down, _) | (KeyCode::Char('j'), _) => {
                if let Some(analysis) = &self.analysis {
                    self.selected_dependency = (self.selected_dependency + 1) % analysis.dependencies.len().max(1);
                }
            }
            (KeyCode::Up, _) | (KeyCode::Char('k'), _) => {
                if let Some(analysis) = &self.analysis {
                    let len = analysis.dependencies.len().max(1);
                    self.selected_dependency = (self.selected_dependency + len - 1) % len;
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
}

/// Run the TUI application
pub fn run(project_path: PathBuf, filter_dep: Option<String>) -> Result<()> {
    // Set up terminal
    terminal::enable_raw_mode()?;
    std::io::stdout().execute(EnterAlternateScreen)?;
    
    // Create terminal backend and terminal
    let backend = CrosstermBackend::new(std::io::stdout());
    let mut terminal = Terminal::new(backend)?;
    
    // Create app state
    let mut app = App::new(project_path, filter_dep);
    
    // Run analysis
    app.run_analysis()?;
    
    // Create event handler
    let event_config = EventConfig {
        tick_rate: Duration::from_millis(250),
    };
    let mut event_handler = EventHandler::new(event_config);
    
    // Main loop
    while !app.should_quit {
        // Draw UI
        terminal.draw(|frame| ui::draw(frame, &app))?;
        
        // Handle events
        match event_handler.next()? {
            Event::Key(key_event) => app.handle_key_event(key_event),
            _ => {}
        }
    }
    
    // Restore terminal
    terminal::disable_raw_mode()?;
    std::io::stdout().execute(LeaveAlternateScreen)?;
    
    Ok(())
} 