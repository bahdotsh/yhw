use ratatui::prelude::*;
use ratatui::style::{Style, Modifier, Color};
use ratatui::widgets::{Block, Borders, Paragraph, Tabs, List, ListItem, Table, Row, Cell};
use ratatui::layout::{Layout, Direction, Constraint};

use crate::analyzer::metrics;
use crate::tui::app::App;

/// Draw the UI
pub fn draw(frame: &mut Frame, app: &App) {
    // Create the layout
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title and tabs
            Constraint::Min(0),    // Content
            Constraint::Length(1), // Status bar
        ])
        .split(frame.size());
    
    // Draw the title and tabs
    draw_title_and_tabs(frame, app, main_layout[0]);
    
    // Draw the content based on the current tab
    match app.current_tab {
        0 => draw_overview_tab(frame, app, main_layout[1]),
        1 => draw_details_tab(frame, app, main_layout[1]),
        2 => draw_removable_tab(frame, app, main_layout[1]),
        _ => {}
    }
    
    // Draw the status bar
    draw_status_bar(frame, app, main_layout[2]);
    
    // Draw help if showing
    if app.show_help {
        draw_help(frame);
    }
}

/// Draw the title and tabs
fn draw_title_and_tabs(frame: &mut Frame, app: &App, area: Rect) {
    let titles = ["Overview", "Details", "Removable"]
        .iter()
        .map(|t| Spans::from(Span::styled(*t, Style::default().fg(Color::White))))
        .collect();
    
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::BOTTOM))
        .select(app.current_tab)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
    
    frame.render_widget(tabs, area);
}

/// Draw the overview tab
fn draw_overview_tab(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(analysis) = &app.analysis {
        // Split the area into the dependency list and the summary
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30), // Dependency list
                Constraint::Percentage(70), // Summary
            ])
            .split(area);
        
        // Create the dependency list
        let deps: Vec<ListItem> = analysis.dependencies
            .iter()
            .enumerate()
            .map(|(i, dep)| {
                let style = if i == app.selected_dependency {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                
                let usage_count = analysis.metrics.usage_count.get(&dep.name).unwrap_or(&0);
                let is_used = analysis.metrics.is_used.get(&dep.name).unwrap_or(&false);
                
                let name = if *is_used {
                    dep.name.clone()
                } else {
                    format!("{} (unused)", dep.name)
                };
                
                ListItem::new(Spans::from(vec![
                    Span::styled(name, style),
                    Span::raw(" "),
                    Span::styled(format!("({})", usage_count), Style::default().fg(Color::DarkGray)),
                ]))
            })
            .collect();
        
        let deps_list = List::new(deps)
            .block(Block::default().title("Dependencies").borders(Borders::ALL))
            .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
        
        frame.render_stateful_widget(deps_list, chunks[0], &mut app.selected_dependency.into());
        
        // Draw the summary
        draw_dependency_summary(frame, app, chunks[1]);
    } else {
        // No analysis results yet
        let message = Paragraph::new("Running analysis...")
            .block(Block::default().title("Analysis").borders(Borders::ALL));
        frame.render_widget(message, area);
    }
}

/// Draw the details tab
fn draw_details_tab(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(analysis) = &app.analysis {
        if let Some(dep) = analysis.dependencies.get(app.selected_dependency) {
            // Get usage data for the selected dependency
            let usages = analysis.usage_data.usage_locations.get(&dep.name).unwrap_or(&Vec::new());
            
            // Create a table of usage locations
            let rows: Vec<Row> = usages.iter()
                .map(|usage| {
                    let file = usage.file.to_string_lossy();
                    let file_path = if file.len() > 40 {
                        format!("...{}", &file[file.len() - 40..])
                    } else {
                        file.to_string()
                    };
                    
                    Row::new(vec![
                        Cell::from(file_path),
                        Cell::from(usage.line.to_string()),
                        Cell::from(usage.imported_item.clone()),
                        Cell::from(format!("{:?}", usage.usage_type)),
                    ])
                })
                .collect();
            
            let header = Row::new(vec![
                Cell::from(Span::styled("File", Style::default().add_modifier(Modifier::BOLD))),
                Cell::from(Span::styled("Line", Style::default().add_modifier(Modifier::BOLD))),
                Cell::from(Span::styled("Item", Style::default().add_modifier(Modifier::BOLD))),
                Cell::from(Span::styled("Type", Style::default().add_modifier(Modifier::BOLD))),
            ]);
            
            let widths = [
                Constraint::Percentage(40),
                Constraint::Percentage(10),
                Constraint::Percentage(30),
                Constraint::Percentage(20),
            ];
            
            let table = Table::new(rows)
                .header(header)
                .block(Block::default().title(format!("Usage of {}", dep.name)).borders(Borders::ALL))
                .widths(&widths)
                .column_spacing(1);
            
            frame.render_widget(table, area);
        }
    } else {
        // No analysis results yet
        let message = Paragraph::new("Running analysis...")
            .block(Block::default().title("Analysis").borders(Borders::ALL));
        frame.render_widget(message, area);
    }
}

/// Draw the removable tab
fn draw_removable_tab(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(analysis) = &app.analysis {
        // Find potentially removable dependencies
        let removable = metrics::find_removable_dependencies(&analysis.metrics);
        
        if removable.is_empty() {
            let message = Paragraph::new("No removable dependencies found!")
                .block(Block::default().title("Removable Dependencies").borders(Borders::ALL));
            frame.render_widget(message, area);
        } else {
            // Create a list of removable dependencies with reasons
            let items: Vec<ListItem> = removable.iter()
                .map(|name| {
                    let is_used = analysis.metrics.is_used.get(name).unwrap_or(&false);
                    let reason = if !is_used {
                        "Unused dependency"
                    } else {
                        "Rarely used dependency"
                    };
                    
                    ListItem::new(vec![
                        Spans::from(Span::styled(name, Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))),
                        Spans::from(Span::raw(format!("  Reason: {}", reason))),
                    ])
                })
                .collect();
            
            let list = List::new(items)
                .block(Block::default().title("Removable Dependencies").borders(Borders::ALL));
            
            frame.render_widget(list, area);
        }
    } else {
        // No analysis results yet
        let message = Paragraph::new("Running analysis...")
            .block(Block::default().title("Analysis").borders(Borders::ALL));
        frame.render_widget(message, area);
    }
}

/// Draw the dependency summary
fn draw_dependency_summary(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(analysis) = &app.analysis {
        if let Some(dep) = analysis.dependencies.get(app.selected_dependency) {
            // Get metrics for the dependency
            let is_used = analysis.metrics.is_used.get(&dep.name).unwrap_or(&false);
            let usage_count = analysis.metrics.usage_count.get(&dep.name).unwrap_or(&0);
            let importance = analysis.metrics.importance_scores.get(&dep.name).unwrap_or(&0.0);
            
            // Format the text
            let text = vec![
                Spans::from(Span::styled(&dep.name, Style::default().add_modifier(Modifier::BOLD))),
                Spans::from(""),
                Spans::from(format!("Version: {}", dep.version.as_deref().unwrap_or("unknown"))),
                Spans::from(format!("Type: {:?}", dep.dependency_type)),
                Spans::from(format!("Optional: {}", dep.optional)),
                Spans::from(""),
                Spans::from(format!("Used: {}", if *is_used { "Yes" } else { "No" })),
                Spans::from(format!("Usage count: {} file(s)", usage_count)),
                Spans::from(format!("Importance: {:.2}", importance)),
                Spans::from(""),
                Spans::from(format!("Features: {}", if dep.features.is_empty() {
                    "None".to_string()
                } else {
                    dep.features.join(", ")
                })),
            ];
            
            let summary = Paragraph::new(text)
                .block(Block::default().title("Dependency Summary").borders(Borders::ALL));
            
            frame.render_widget(summary, area);
        }
    }
}

/// Draw the status bar
fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let status = format!("Project: {} | Press ? for help", app.project_path.display());
    let status_bar = Paragraph::new(status)
        .style(Style::default().fg(Color::White).bg(Color::Blue));
    
    frame.render_widget(status_bar, area);
}

/// Draw the help dialog
fn draw_help(frame: &mut Frame) {
    let area = centered_rect(60, 20, frame.size());
    
    let help_text = vec![
        Spans::from(Span::styled("Help", Style::default().add_modifier(Modifier::BOLD))),
        Spans::from(""),
        Spans::from("q, Esc: Quit"),
        Spans::from("Tab: Next tab"),
        Spans::from("Shift+Tab: Previous tab"),
        Spans::from("Up/Down, j/k: Navigate dependencies"),
        Spans::from("?: Toggle help"),
        Spans::from(""),
        Spans::from("Press any key to close help"),
    ];
    
    let help = Paragraph::new(help_text)
        .block(Block::default().title("Help").borders(Borders::ALL))
        .style(Style::default().fg(Color::White).bg(Color::Black));
    
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
        ])
        .split(r);
    
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
} 