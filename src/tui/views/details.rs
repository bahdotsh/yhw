use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Table, Row, Cell, Tabs};
use ratatui::Frame;

use crate::analyzer::{AnalysisResult, DependencyUsage, UsageType};
use crate::tui::app::App;

/// Render the details view for a selected dependency
pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    // Create layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Length(3),  // Detail view tabs
            Constraint::Min(0),     // Detail content
        ].as_ref())
        .split(area);
    
    // Get the actual dependency index based on filtered view
    let actual_idx = app.actual_selected_index();
    
    // Render title
    let title = if let Some(analysis) = &app.analysis {
        if let Some(dep_idx) = actual_idx {
            if let Some(dep) = analysis.dependencies.get(dep_idx) {
                format!("Dependency Details: {}", dep.name)
            } else {
                "Dependency Details".to_string()
            }
        } else {
            "Dependency Details".to_string()
        }
    } else {
        "Dependency Details".to_string()
    };
    
    let title_widget = Paragraph::new(Line::from(vec![
        Span::styled(title, Style::default().add_modifier(Modifier::BOLD))
    ]))
    .block(Block::default()
        .borders(Borders::ALL)
        .style(Style::default()));
    
    frame.render_widget(title_widget, chunks[0]);
    
    // Draw detail view tabs
    let detail_titles = vec!["Basic Info", "Usage Metrics", "Dependencies"];
    let detail_tabs = Tabs::new(detail_titles.iter().map(|t| Line::from(Span::styled(*t, Style::default().fg(Color::White)))).collect())
        .block(Block::default().borders(Borders::ALL))
        .select(app.detail_view)
        .style(Style::default().fg(Color::Cyan))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    
    frame.render_widget(detail_tabs, chunks[1]);
    
    if let Some(analysis) = &app.analysis {
        if let Some(dep_idx) = actual_idx {
            if let Some(dep) = analysis.dependencies.get(dep_idx) {
                // Render the appropriate detail view
                match app.detail_view {
                    0 => render_basic_info(frame, app, analysis, dep, chunks[2]),
                    1 => render_usage_metrics(frame, app, analysis, dep, chunks[2]),
                    2 => render_dependency_graph_info(frame, app, analysis, &dep.name, chunks[2]),
                    _ => {}
                }
            } else {
                // No dependency selected
                let message = Paragraph::new("No dependency selected")
                    .block(Block::default().borders(Borders::ALL));
                frame.render_widget(message, chunks[2]);
            }
        } else {
            // No dependency selected (may happen with filtering)
            let message = Paragraph::new("No dependencies match the current filter")
                .block(Block::default().borders(Borders::ALL));
            frame.render_widget(message, chunks[2]);
        }
    } else {
        // Loading message
        let loading = Paragraph::new("Loading dependency details...")
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(loading, chunks[2]);
    }
}

/// Render basic information about a dependency
fn render_basic_info(frame: &mut Frame, _app: &App, analysis: &AnalysisResult, dep: &crate::manifest::cargo::CargoDependency, area: Rect) {
    // Split the area for basic info and features
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ].as_ref())
        .split(area);
    
    // Render basic dependency info in the left section
    let is_used = analysis.metrics.is_used.get(&dep.name).unwrap_or(&false);
    let usage_count = analysis.metrics.usage_count.get(&dep.name).unwrap_or(&0);
    let importance = analysis.metrics.importance_scores.get(&dep.name).unwrap_or(&0.0);
    let is_partially_used = analysis.metrics.is_partially_used.get(&dep.name).unwrap_or(&false);
    let is_removable = analysis.metrics.removable_dependencies.contains(&dep.name);
    
    let info_text = vec![
        Line::from(vec![
            Span::styled("Name: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&dep.name)
        ]),
        Line::from(vec![
            Span::styled("Version: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(dep.version.as_deref().unwrap_or("unknown"))
        ]),
        Line::from(vec![
            Span::styled("Type: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!("{:?}", dep.dependency_type))
        ]),
        Line::from(vec![
            Span::styled("Optional: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!("{}", dep.optional))
        ]),
        Line::from(vec![
            Span::styled("Used: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                if *is_used { "Yes" } else { "No" }, 
                Style::default().fg(if *is_used { Color::Green } else { Color::Red })
            )
        ]),
        Line::from(vec![
            Span::styled("Usage count: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!("{} file(s)", usage_count))
        ]),
        Line::from(vec![
            Span::styled("Importance: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("{:.2}", importance),
                Style::default().fg(importance_color(*importance))
            )
        ]),
        Line::from(vec![
            Span::styled("Partially used: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(if *is_partially_used { "Yes" } else { "No" })
        ]),
        Line::from(vec![
            Span::styled("Removable: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                if is_removable { "Yes" } else { "No" },
                Style::default().fg(if is_removable { Color::Red } else { Color::Green })
            )
        ]),
    ];
    
    let info = Paragraph::new(info_text)
        .block(Block::default().borders(Borders::ALL).title("Basic Information"));
    
    frame.render_widget(info, chunks[0]);
    
    // Render features in the right section
    let mut feature_text = Vec::new();
    
    feature_text.push(Line::from(vec![
        Span::styled("Features:", Style::default().add_modifier(Modifier::BOLD))
    ]));
    
    if dep.features.is_empty() {
        feature_text.push(Line::from("  None"));
    } else {
        // Get feature usage if available
        // Use a static empty map to avoid temporary value issues
        static EMPTY_FEATURE_USAGE: std::sync::OnceLock<std::collections::HashMap<String, bool>> = std::sync::OnceLock::new();
        let feature_usage_map = analysis.metrics.feature_usage.get(&dep.name)
            .unwrap_or_else(|| EMPTY_FEATURE_USAGE.get_or_init(|| std::collections::HashMap::new()));
        
        for feature in &dep.features {
            let is_used = feature_usage_map.get(feature).unwrap_or(&false);
            let is_used_val = *is_used; // Dereference once to avoid borrowing issue
            feature_text.push(Line::from(vec![
                Span::raw(format!("  {}: ", feature)),
                Span::styled(
                    if is_used_val { "Used" } else { "Unused" },
                    Style::default().fg(if is_used_val { Color::Green } else { Color::Red })
                )
            ]));
        }
    }
    
    let features = Paragraph::new(feature_text)
        .block(Block::default().borders(Borders::ALL).title("Features"));
    
    frame.render_widget(features, chunks[1]);
}

/// Render usage metrics for a dependency
fn render_usage_metrics(frame: &mut Frame, _app: &App, analysis: &AnalysisResult, dep: &crate::manifest::cargo::CargoDependency, area: Rect) {
    // Create static empty maps to use as fallbacks
    static EMPTY_USAGE_TYPES: std::sync::OnceLock<std::collections::HashMap<UsageType, usize>> = std::sync::OnceLock::new();
    static EMPTY_FEATURE_USAGE: std::sync::OnceLock<std::collections::HashMap<String, bool>> = std::sync::OnceLock::new();
    
    // Split the area for usage metrics and usage locations
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(10),  // Usage types and metrics
            Constraint::Min(0),      // Usage locations
        ].as_ref())
        .split(area);
    
    // Get usage types
    let usage_types = analysis.metrics.usage_types.get(&dep.name)
        .unwrap_or_else(|| EMPTY_USAGE_TYPES.get_or_init(|| std::collections::HashMap::new()));
    
    // Get feature usage
    let feature_usage = analysis.metrics.feature_usage.get(&dep.name)
        .unwrap_or_else(|| EMPTY_FEATURE_USAGE.get_or_init(|| std::collections::HashMap::new()));
    
    // Split the top area for usage types and feature usage
    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ].as_ref())
        .split(chunks[0]);
    
    // Render usage types in left section
    let mut usage_type_text = Vec::new();
    
    usage_type_text.push(Line::from(vec![
        Span::styled("Usage Types:", Style::default().add_modifier(Modifier::BOLD))
    ]));
    
    if usage_types.is_empty() {
        usage_type_text.push(Line::from("  None"));
    } else {
        for (usage_type, count) in usage_types {
            usage_type_text.push(Line::from(vec![
                Span::raw(format!("  {:?}: {}", usage_type, count))
            ]));
        }
    }
    
    let usage_type_widget = Paragraph::new(usage_type_text)
        .block(Block::default().borders(Borders::ALL).title("Usage Types"));
    
    frame.render_widget(usage_type_widget, top_chunks[0]);
    
    // Render feature usage in right section
    let mut feature_text = Vec::new();
    
    feature_text.push(Line::from(vec![
        Span::styled("Feature Usage:", Style::default().add_modifier(Modifier::BOLD))
    ]));
    
    if feature_usage.is_empty() {
        feature_text.push(Line::from("  No features"));
    } else {
        for (feature, is_used) in feature_usage {
            feature_text.push(Line::from(vec![
                Span::raw(format!("  {}: ", feature)),
                Span::styled(
                    if *is_used { "Used" } else { "Unused" },
                    Style::default().fg(if *is_used { Color::Green } else { Color::Red })
                )
            ]));
        }
    }
    
    let feature_widget = Paragraph::new(feature_text)
        .block(Block::default().borders(Borders::ALL).title("Feature Usage"));
    
    frame.render_widget(feature_widget, top_chunks[1]);
    
    // Render usage locations in bottom section
    render_usage_locations(frame, analysis, &dep.name, chunks[1]);
}

/// Render dependency graph information
fn render_dependency_graph_info(frame: &mut Frame, _app: &App, analysis: &AnalysisResult, dep_name: &str, area: Rect) {
    // Split the area
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(10),  // Metrics
            Constraint::Min(0),      // Circular dependencies
        ].as_ref())
        .split(area);
    
    // In our implementation, we need to calculate dependencies from the graph structure
    // Count the number of nodes in the graph
    let node_count = analysis.dependency_graph.graph.node_count();
    
    // Check if this dependency is in any circular dependencies
    let circular_deps = analysis.dependency_graph.find_circular_dependencies();
    let dep_name_owned = dep_name.to_string();
    
    // Clone the circular_deps vector to avoid borrowing issues
    let circular_deps_owned: Vec<Vec<String>> = circular_deps.clone();
    
    // Now check if this dependency is in any circular dependencies
    let is_in_circular = circular_deps_owned.iter().any(|cycle| cycle.contains(&dep_name_owned));
    
    // Create a summary of the dependency graph
    let graph_info = vec![
        Line::from(vec![
            Span::styled("Dependency Graph:", Style::default().add_modifier(Modifier::BOLD))
        ]),
        Line::from(vec![
            Span::raw(format!("Total dependencies in graph: {}", node_count))
        ]),
        Line::from(vec![
            Span::raw("In circular dependency: "),
            Span::styled(
                if is_in_circular { "Yes" } else { "No" },
                Style::default().fg(if is_in_circular { Color::Red } else { Color::Green })
            )
        ]),
    ];
    
    let graph_widget = Paragraph::new(graph_info)
        .block(Block::default().borders(Borders::ALL).title("Dependency Graph Info"));
    
    frame.render_widget(graph_widget, chunks[0]);
    
    // Show circular dependencies if this dependency is in any
    if is_in_circular {
        let circular_text: Vec<Line> = circular_deps_owned.iter()
            .filter(|cycle| cycle.contains(&dep_name_owned))
            .map(|cycle| {
                Line::from(vec![
                    Span::raw(format!("• {}", cycle.join(" → ")))
                ])
            })
            .collect();
        
        let circular_widget = Paragraph::new(circular_text)
            .block(Block::default().borders(Borders::ALL).title("Circular Dependencies"));
        
        frame.render_widget(circular_widget, chunks[1]);
    } else {
        let no_circular = Paragraph::new("This dependency is not part of any circular dependencies.")
            .block(Block::default().borders(Borders::ALL).title("Circular Dependencies"));
        
        frame.render_widget(no_circular, chunks[1]);
    }
}

/// Get color for importance score
fn importance_color(score: f64) -> Color {
    if score >= 0.7 {
        Color::Green
    } else if score >= 0.3 {
        Color::Yellow
    } else {
        Color::Red
    }
}

/// Render usage locations for a dependency
fn render_usage_locations(frame: &mut Frame, analysis: &AnalysisResult, dep_name: &str, area: Rect) {
    // Get usage information for the dependency
    let empty_vec = Vec::new();
    let usage_locations = analysis.usage_data.usage_locations.get(dep_name).unwrap_or(&empty_vec);
    
    if !usage_locations.is_empty() {
        // Create a list of usage locations
        let items: Vec<ListItem> = usage_locations.iter()
            .map(|usage| {
                let mut lines = vec![
                    Line::from(vec![
                        Span::styled(
                            usage.file.to_string_lossy().to_string(),
                            Style::default().add_modifier(Modifier::BOLD)
                        ),
                        Span::raw(format!(" (line {})", usage.line))
                    ]),
                    Line::from(vec![
                        Span::raw(format!("  Import: {}", usage.imported_item)),
                    ]),
                    Line::from(vec![
                        Span::raw(format!("  Type: {:?}", usage.usage_type))
                    ])
                ];
                
                ListItem::new(lines)
            })
            .collect();
        
        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(format!("Usage Locations ({})", usage_locations.len())))
            .style(Style::default().fg(Color::White));
        
        frame.render_widget(list, area);
    } else {
        // No usage information
        let no_usage = Paragraph::new("No usage information available")
            .block(Block::default().borders(Borders::ALL).title("Usage Locations"));
        
        frame.render_widget(no_usage, area);
    }
} 