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
            Constraint::Length(3),  // Title and tabs
            Constraint::Length(3),  // Status bar with sort/filter info
            Constraint::Min(0),     // Main content
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
    
    // Draw status bar with sort/filter info
    draw_status_bar(frame, app, chunks[1]);
    
    // Draw content based on selected tab
    match app.current_tab {
        0 => draw_overview_tab(frame, app, chunks[2]),
        1 => draw_details_tab(frame, app, chunks[2]),
        2 => draw_removable_tab(frame, app, chunks[2]),
        _ => {}
    }
    
    // Draw search bar if in search mode
    if app.is_searching {
        draw_search_bar(frame, app);
    }
    
    // Draw help popup if needed
    if app.show_help {
        draw_help(frame);
    }
}

/// Draw the status bar with sorting and filtering information
fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let mut status_text = Vec::new();
    
    // Add the sort information
    status_text.push(Span::styled("Sort: ", Style::default().fg(Color::Blue)));
    status_text.push(Span::styled(
        app.sort_option.as_str(), 
        Style::default().fg(Color::Yellow)
    ));
    
    // Add the sort direction
    status_text.push(Span::raw(" "));
    status_text.push(Span::styled(
        if app.sort_reverse { "↑" } else { "↓" },
        Style::default().fg(Color::Yellow)
    ));
    
    // Add separator
    status_text.push(Span::raw(" | "));
    
    // Add the filter information
    status_text.push(Span::styled("Filter: ", Style::default().fg(Color::Blue)));
    status_text.push(Span::styled(
        app.filter_option.as_str(),
        Style::default().fg(Color::Yellow)
    ));
    
    // Add search query if applicable
    if !app.search_query.is_empty() {
        status_text.push(Span::raw(" | "));
        status_text.push(Span::styled("Search: ", Style::default().fg(Color::Blue)));
        status_text.push(Span::styled(
            &app.search_query,
            Style::default().fg(Color::Yellow)
        ));
    }
    
    // Add controls hint
    status_text.push(Span::raw(" | "));
    status_text.push(Span::styled("(s)ort", Style::default().fg(Color::Blue)));
    status_text.push(Span::raw(" | "));
    status_text.push(Span::styled("(r)everse", Style::default().fg(Color::Blue)));
    status_text.push(Span::raw(" | "));
    status_text.push(Span::styled("(f)ilter", Style::default().fg(Color::Blue)));
    status_text.push(Span::raw(" | "));
    status_text.push(Span::styled("(/)search", Style::default().fg(Color::Blue)));
    status_text.push(Span::raw(" | "));
    status_text.push(Span::styled("(?)help", Style::default().fg(Color::Blue)));
    
    let status = Paragraph::new(Line::from(status_text))
        .block(Block::default().borders(Borders::ALL).title("Status"));
    
    frame.render_widget(status, area);
}

/// Draw search bar popup
fn draw_search_bar(frame: &mut Frame, app: &App) {
    let area = centered_rect(50, 20, frame.size());
    
    let search_text = format!("Search: {}", app.search_query);
    let search_bar = Paragraph::new(search_text)
        .block(Block::default().borders(Borders::ALL).title("Search Dependencies"))
        .style(Style::default().fg(Color::White));
    
    // Clear the area first
    frame.render_widget(Clear, area);
    frame.render_widget(search_bar, area);
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
        let filtered_indices = app.filtered_dependencies();
        
        // Filter for removable dependencies
        let removable_indices: Vec<_> = filtered_indices.into_iter()
            .filter(|&i| {
                let dep_name = &analysis.dependencies[i].name;
                analysis.metrics.removable_dependencies.contains(dep_name)
            })
            .collect();
        
        let items: Vec<ListItem> = if !removable_indices.is_empty() {
            removable_indices.iter()
                .map(|&i| {
                    let dep = &analysis.dependencies[i];
                    let dep_name = &dep.name;
                    
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
                    let dep_info = {
                        let dep_type = match dep.dependency_type {
                            DependencyType::Normal => "normal",
                            DependencyType::Development => "dev",
                            DependencyType::Build => "build",
                        };
                        
                        let optional = if dep.optional { ", optional" } else { "" };
                        format!("{} dependency{}", dep_type, optional)
                    };
                    
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
        Line::from("Left/Right, h/l: Navigate views in details tab"),
        Line::from("s: Cycle sort options"),
        Line::from("r: Reverse sort order"),
        Line::from("f: Cycle filter options"),
        Line::from("/: Search dependencies"),
        Line::from("?: Toggle help"),
        Line::from(""),
        Line::from("Press Esc to close help or search"),
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