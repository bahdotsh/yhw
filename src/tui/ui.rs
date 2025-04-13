use std::collections::HashMap;

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Tabs, Clear, Table, Row, Cell};
use ratatui::Frame;

use crate::tui::app::App;
use crate::analyzer::{AnalysisResult, DependencyUsage};
use crate::manifest::cargo::DependencyType;

/// Draw the UI
pub fn draw(frame: &mut Frame, app: &App) {
    // Create main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
        ].as_ref())
        .split(frame.size());
    
    // Draw title
    let title = Paragraph::new(Line::from(vec![
        Span::styled("Why", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw(" - Dependency Analysis Tool"),
    ]))
    .block(Block::default()
        .borders(Borders::ALL)
        .style(Style::default()));
    
    frame.render_widget(title, chunks[0]);
    
    // Draw tabs
    let titles = vec!["Overview", "Details", "Removable"];
    let tabs = Tabs::new(titles.iter().map(|t| Line::from(Span::styled(*t, Style::default().fg(Color::White)))).collect())
        .block(Block::default().borders(Borders::ALL))
        .select(app.current_tab)
        .style(Style::default().fg(Color::Cyan))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    
    frame.render_widget(tabs, chunks[0]);
    
    // Draw content based on selected tab
    match app.current_tab {
        0 => draw_overview_tab(frame, app, chunks[1]),
        1 => draw_details_tab(frame, app, chunks[1]),
        2 => draw_removable_tab(frame, app, chunks[1]),
        _ => {}
    }
    
    // Draw help popup if needed
    if app.show_help {
        draw_help(frame);
    }
}

/// Draw the overview tab
fn draw_overview_tab(frame: &mut Frame, app: &App, area: Rect) {
    // If we have analysis results, delegate to the overview view
    if app.analysis.is_some() {
        crate::tui::views::overview::render(frame, app, area);
    } else {
        // Otherwise show a loading message
        let loading = Paragraph::new("Loading analysis...").block(Block::default().borders(Borders::ALL));
        frame.render_widget(loading, area);
    }
}

/// Draw the details tab
fn draw_details_tab(frame: &mut Frame, app: &App, area: Rect) {
    // If we have analysis results, delegate to the details view
    if app.analysis.is_some() {
        crate::tui::views::details::render(frame, app, area);
    } else {
        // Otherwise show a loading message
        let loading = Paragraph::new("Loading analysis...").block(Block::default().borders(Borders::ALL));
        frame.render_widget(loading, area);
    }
}

/// Draw the removable dependencies tab
fn draw_removable_tab(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(analysis) = &app.analysis {
        // Use the calculated removable dependencies list
        let items: Vec<ListItem> = if !analysis.metrics.removable_dependencies.is_empty() {
            analysis.metrics.removable_dependencies.iter()
                .map(|dep_name| {
                    let reason = if !analysis.metrics.is_used.get(dep_name).unwrap_or(&true) {
                        "Dependency is not used in the codebase"
                    } else if let Some(true) = analysis.metrics.is_partially_used.get(dep_name) {
                        "Dependency is only partially used (unused features)"
                    } else {
                        let score = analysis.metrics.importance_scores.get(dep_name).unwrap_or(&0.0);
                        if *score < 0.1 {
                            "Dependency has very low usage"
                        } else {
                            "Low overall importance to the project"
                        }
                    };
                    
                    // Get more details about the dependency
                    let dep_info = analysis.dependencies.iter()
                        .find(|d| &d.name == dep_name)
                        .map(|d| {
                            let dep_type = match d.dependency_type {
                                DependencyType::Normal => "normal",
                                DependencyType::Development => "dev",
                                DependencyType::Build => "build",
                            };
                            
                            let optional = if d.optional { ", optional" } else { "" };
                            format!("{} dependency{}", dep_type, optional)
                        })
                        .unwrap_or_else(|| "Unknown dependency type".to_string());
                    
                    // Get usage count
                    let usage_count = analysis.metrics.usage_count.get(dep_name).unwrap_or(&0);
                    
                    ListItem::new(vec![
                        Line::from(vec![
                            Span::styled(dep_name, Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
                        ]),
                        Line::from(vec![
                            Span::raw(format!("  Reason: {}", reason))
                        ]),
                        Line::from(vec![
                            Span::raw(format!("  Type: {}", dep_info))
                        ]),
                        Line::from(vec![
                            Span::raw(format!("  Used in {} file(s)", usage_count))
                        ]),
                    ])
                })
                .collect()
        } else {
            vec![ListItem::new("No dependencies identified as removable")]
        };
        
        // Create the list
        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Dependencies that might be removable"))
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol("> ");
        
        frame.render_widget(list, area);
    } else {
        // Show loading message
        let loading = Paragraph::new("Loading analysis...").block(Block::default().borders(Borders::ALL));
        frame.render_widget(loading, area);
    }
}

/// Draw help popup
fn draw_help(frame: &mut Frame) {
    let area = centered_rect(60, 70, frame.size());
    
    let help_text = vec![
        Line::from(vec![
            Span::styled("Help", Style::default().add_modifier(Modifier::BOLD))
        ]),
        Line::from(""),
        Line::from("q, Esc: Quit"),
        Line::from("Tab: Next tab"),
        Line::from("Shift+Tab: Previous tab"),
        Line::from("Up/Down, j/k: Navigate dependencies"),
        Line::from("?: Toggle help"),
        Line::from(""),
        Line::from("Press any key to close help"),
    ];
    
    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .style(Style::default().fg(Color::White));
    
    // Clear the area first
    frame.render_widget(Clear, area);
    frame.render_widget(help, area);
}

/// Helper function to create a centered rect
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ].as_ref())
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ].as_ref())
        .split(popup_layout[1])[1]
} 