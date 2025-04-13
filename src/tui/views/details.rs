use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Table, Row, Cell};
use ratatui::Frame;

use crate::analyzer::{AnalysisResult, DependencyUsage, UsageType};
use crate::tui::app::App;

/// Render the details view for a selected dependency
pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    // Create layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(12), // Increased to fit more info
            Constraint::Min(0),
        ].as_ref())
        .split(area);
    
    // Render title
    let title = if let Some(analysis) = &app.analysis {
        if let Some(dep) = analysis.dependencies.get(app.selected_dependency) {
            format!("Dependency Details: {}", dep.name)
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
    
    if let Some(analysis) = &app.analysis {
        if let Some(dep) = analysis.dependencies.get(app.selected_dependency) {
            // Split the main area for different sections
            let info_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(50),
                    Constraint::Percentage(50),
                ].as_ref())
                .split(chunks[1]);
            
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
                Line::from(vec![
                    Span::styled("Features: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(if dep.features.is_empty() { 
                        "None".to_string() 
                    } else { 
                        dep.features.join(", ") 
                    })
                ]),
            ];
            
            let info = Paragraph::new(info_text)
                .block(Block::default().borders(Borders::ALL).title("Basic Information"));
            
            frame.render_widget(info, info_chunks[0]);
            
            // Render usage metrics in the right section
            render_usage_metrics(frame, analysis, dep, info_chunks[1]);
            
            // Split the bottom area for usage locations and dependency graph
            let bottom_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(70),
                    Constraint::Percentage(30),
                ].as_ref())
                .split(chunks[2]);
            
            // Render usage locations in top part
            render_usage_locations(frame, analysis, dep.name.as_str(), bottom_chunks[0]);
            
            // Render dependency graph info in bottom part
            render_dependency_graph_info(frame, analysis, dep.name.as_str(), bottom_chunks[1]);
        }
    } else {
        let loading = Paragraph::new("Loading dependency details...")
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(loading, chunks[1]);
    }
}

/// Render usage metrics for a dependency
fn render_usage_metrics(frame: &mut Frame, analysis: &AnalysisResult, dep: &crate::manifest::cargo::CargoDependency, area: Rect) {
    // Create static empty maps to use as fallbacks
    static EMPTY_USAGE_TYPES: std::sync::OnceLock<std::collections::HashMap<UsageType, usize>> = std::sync::OnceLock::new();
    static EMPTY_FEATURE_USAGE: std::sync::OnceLock<std::collections::HashMap<String, bool>> = std::sync::OnceLock::new();
    
    // Get usage types
    let usage_types = analysis.metrics.usage_types.get(&dep.name)
        .unwrap_or_else(|| EMPTY_USAGE_TYPES.get_or_init(|| std::collections::HashMap::new()));
    
    // Get feature usage
    let feature_usage = analysis.metrics.feature_usage.get(&dep.name)
        .unwrap_or_else(|| EMPTY_FEATURE_USAGE.get_or_init(|| std::collections::HashMap::new()));
    
    let mut metrics_text = Vec::new();
    
    // Add usage type metrics
    metrics_text.push(Line::from(vec![
        Span::styled("Usage Types:", Style::default().add_modifier(Modifier::BOLD))
    ]));
    
    if usage_types.is_empty() {
        metrics_text.push(Line::from("  None"));
    } else {
        for (usage_type, count) in usage_types {
            metrics_text.push(Line::from(vec![
                Span::raw(format!("  {:?}: {}", usage_type, count))
            ]));
        }
    }
    
    // Add feature usage metrics
    metrics_text.push(Line::from(vec![
        Span::styled("Feature Usage:", Style::default().add_modifier(Modifier::BOLD))
    ]));
    
    if feature_usage.is_empty() {
        metrics_text.push(Line::from("  No features"));
    } else {
        for (feature, is_used) in feature_usage {
            metrics_text.push(Line::from(vec![
                Span::raw(format!("  {}: ", feature)),
                Span::styled(
                    if *is_used { "Used" } else { "Unused" },
                    Style::default().fg(if *is_used { Color::Green } else { Color::Red })
                )
            ]));
        }
    }
    
    let metrics = Paragraph::new(metrics_text)
        .block(Block::default().borders(Borders::ALL).title("Usage Metrics"));
    
    frame.render_widget(metrics, area);
}

/// Render dependency graph information
fn render_dependency_graph_info(frame: &mut Frame, analysis: &AnalysisResult, dep_name: &str, area: Rect) {
    // Count the number of nodes in the graph
    let node_count = analysis.dependency_graph.graph.node_count();
    
    // Check if this dependency is in any circular dependencies
    let circular_deps = analysis.dependency_graph.find_circular_dependencies();
    let dep_name_owned = dep_name.to_string(); // Create owned version to avoid temporary value issues
    let is_in_circular = circular_deps.iter().any(|cycle| cycle.contains(&dep_name_owned));
    
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
    
    frame.render_widget(graph_widget, area);
}

/// Map importance score to a color
fn importance_color(score: f64) -> Color {
    if score < 0.3 {
        Color::Red
    } else if score < 0.6 {
        Color::Yellow
    } else {
        Color::Green
    }
}

/// Render the usage locations for a dependency
fn render_usage_locations(frame: &mut Frame, analysis: &AnalysisResult, dep_name: &str, area: Rect) {
    let empty_vec = Vec::new();
    let usages = analysis.usage_data.usage_locations.get(dep_name).unwrap_or(&empty_vec);
    
    if usages.is_empty() {
        let no_usages = Paragraph::new("No usages found for this dependency.")
            .block(Block::default().borders(Borders::ALL).title("Usage Locations"));
        frame.render_widget(no_usages, area);
        return;
    }
    
    // Create table rows
    let rows: Vec<Row> = usages.iter()
        .map(|usage| {
            let file = usage.file.to_string_lossy();
            let cells = vec![
                Cell::from(format!("{}", usage.line)),
                Cell::from(format!("{}", file)),
                Cell::from(format!("{}", usage.imported_item)),
                Cell::from(format!("{:?}", usage.usage_type)),
            ];
            Row::new(cells)
        })
        .collect();
    
    // Create table
    let table = Table::new(rows)
        .header(Row::new(vec![
            Cell::from(Span::styled("Line", Style::default().add_modifier(Modifier::BOLD))),
            Cell::from(Span::styled("File", Style::default().add_modifier(Modifier::BOLD))),
            Cell::from(Span::styled("Item", Style::default().add_modifier(Modifier::BOLD))),
            Cell::from(Span::styled("Type", Style::default().add_modifier(Modifier::BOLD))),
        ]))
        .block(Block::default().borders(Borders::ALL).title("Usage Locations"))
        .widths(&[
            Constraint::Length(6),  // Line number
            Constraint::Percentage(40), // File path
            Constraint::Percentage(40), // Imported item
            Constraint::Length(10), // Usage type
        ]);
    
    frame.render_widget(table, area);
} 