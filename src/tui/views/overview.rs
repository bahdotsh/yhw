use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Gauge};
use ratatui::Frame;

use crate::analyzer::AnalysisResult;
use crate::tui::app::App;
use crate::manifest::cargo::DependencyType;

/// Render the overview view
pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    // Create layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(6), // Summary section
            Constraint::Min(0),
        ].as_ref())
        .split(area);
    
    // Render title
    let title = Paragraph::new(Line::from(vec![
        Span::styled("Dependencies Overview", Style::default().add_modifier(Modifier::BOLD))
    ]))
    .block(Block::default()
        .borders(Borders::ALL)
        .style(Style::default()));
    
    frame.render_widget(title, chunks[0]);
    
    if let Some(analysis) = &app.analysis {
        // Render the summary
        render_dependency_summary(frame, analysis, chunks[1]);
        
        // Render the dependency list
        render_dependencies_list(frame, app, analysis, chunks[2]);
    } else {
        let loading = Paragraph::new("Loading dependencies...").block(Block::default().borders(Borders::ALL));
        frame.render_widget(loading, chunks[1]);
    }
}

/// Render a summary of the dependency analysis
fn render_dependency_summary(frame: &mut Frame, analysis: &AnalysisResult, area: Rect) {
    // Split the area horizontally for different metrics
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ].as_ref())
        .split(area);
    
    // Count dependencies by type
    let normal_deps = analysis.dependencies.iter()
        .filter(|d| d.dependency_type == DependencyType::Normal)
        .count();
    
    let dev_deps = analysis.dependencies.iter()
        .filter(|d| d.dependency_type == DependencyType::Development)
        .count();
    
    let build_deps = analysis.dependencies.iter()
        .filter(|d| d.dependency_type == DependencyType::Build)
        .count();
    
    let total_deps = analysis.dependencies.len();
    
    // Count used vs unused dependencies
    let used_deps = analysis.metrics.is_used.values().filter(|&&used| used).count();
    let unused_deps = total_deps - used_deps;
    
    // Potentially removable dependencies
    let removable_deps = analysis.metrics.removable_dependencies.len();
    
    // Calculate usage ratio
    let used_ratio = if total_deps > 0 {
        (used_deps as f64) / (total_deps as f64)
    } else {
        0.0
    };
    
    // Render counts section
    let count_text = vec![
        Line::from(vec![
            Span::styled("Total Dependencies: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!("{}", total_deps))
        ]),
        Line::from(vec![
            Span::raw(format!(" • Normal: {}", normal_deps))
        ]),
        Line::from(vec![
            Span::raw(format!(" • Development: {}", dev_deps))
        ]),
        Line::from(vec![
            Span::raw(format!(" • Build: {}", build_deps))
        ]),
    ];
    
    let count_paragraph = Paragraph::new(count_text)
        .block(Block::default().borders(Borders::ALL).title("Dependency Counts"));
    
    frame.render_widget(count_paragraph, chunks[0]);
    
    // Render usage section
    let usage_text = vec![
        Line::from(vec![
            Span::styled("Used: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("{}", used_deps),
                Style::default().fg(Color::Green)
            )
        ]),
        Line::from(vec![
            Span::styled("Unused: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("{}", unused_deps),
                Style::default().fg(if unused_deps > 0 { Color::Red } else { Color::Green })
            )
        ]),
        Line::from(vec![
            Span::styled("Potentially Removable: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("{}", removable_deps),
                Style::default().fg(if removable_deps > 0 { Color::Yellow } else { Color::Green })
            )
        ]),
    ];
    
    let usage_paragraph = Paragraph::new(usage_text)
        .block(Block::default().borders(Borders::ALL).title("Usage Summary"));
    
    frame.render_widget(usage_paragraph, chunks[1]);
    
    // Render usage gauge
    let usage_gauge = Gauge::default()
        .block(Block::default().title("Usage Ratio").borders(Borders::ALL))
        .gauge_style(Style::default().fg(usage_gauge_color(used_ratio)))
        .percent((used_ratio * 100.0) as u16);
    
    frame.render_widget(usage_gauge, chunks[2]);
}

/// Get color for usage gauge based on ratio
fn usage_gauge_color(ratio: f64) -> Color {
    if ratio >= 0.8 {
        Color::Green
    } else if ratio >= 0.6 {
        Color::Yellow
    } else {
        Color::Red
    }
}

/// Render the list of dependencies
fn render_dependencies_list(frame: &mut Frame, app: &App, analysis: &AnalysisResult, area: Rect) {
    let deps: Vec<ListItem> = analysis.dependencies
        .iter()
        .enumerate()
        .map(|(i, dep)| {
            let style = if i == app.selected_dependency {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            
            let used = analysis.metrics.is_used.get(&dep.name).unwrap_or(&false);
            let importance = analysis.metrics.importance_scores.get(&dep.name).unwrap_or(&0.0);
            let is_removable = analysis.metrics.removable_dependencies.contains(&dep.name);
            
            // Show dependency name with color based on importance
            let name_style = if *used {
                if *importance > 0.7 {
                    Style::default().fg(Color::Green)
                } else if *importance > 0.3 {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::Red)
                }
            } else {
                Style::default().fg(Color::Gray)
            };
            
            // Show type indicator
            let type_indicator = match dep.dependency_type {
                DependencyType::Normal => "",
                DependencyType::Development => " [dev]",
                DependencyType::Build => " [build]",
            };
            
            // Show removable indicator
            let removable_indicator = if is_removable { " ✕" } else { "" };
            
            ListItem::new(Line::from(vec![
                Span::styled(&dep.name, name_style),
                Span::raw(type_indicator),
                Span::styled(removable_indicator, Style::default().fg(Color::Red)),
                Span::raw(" "),
                Span::styled(format!("({:.2})", importance), Style::default().fg(Color::Gray))
            ])).style(style)
        })
        .collect();
    
    let deps_list = List::new(deps)
        .block(Block::default().borders(Borders::ALL).title(format!("Dependencies ({})", analysis.dependencies.len())))
        .highlight_style(Style::default().bg(Color::DarkGray))
        .highlight_symbol("> ");
    
    // Create a mutable list state based on selected dependency
    let mut list_state = ratatui::widgets::ListState::default();
    list_state.select(Some(app.selected_dependency));
    
    frame.render_stateful_widget(deps_list, area, &mut list_state);
} 