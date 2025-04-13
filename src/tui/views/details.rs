use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Table, Row, Cell};
use ratatui::Frame;

use crate::analyzer::{AnalysisResult, DependencyUsage};
use crate::tui::app::App;

/// Render the details view for a selected dependency
pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    // Create layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(8),
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
            // Render dependency info
            let is_used = analysis.metrics.is_used.get(&dep.name).unwrap_or(&false);
            let usage_count = analysis.metrics.usage_count.get(&dep.name).unwrap_or(&0);
            let importance = analysis.metrics.importance_scores.get(&dep.name).unwrap_or(&0.0);
            
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
                    Span::raw(if *is_used { "Yes" } else { "No" })
                ]),
                Line::from(vec![
                    Span::styled("Usage count: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(format!("{} file(s)", usage_count))
                ]),
                Line::from(vec![
                    Span::styled("Importance: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(format!("{:.2}", importance))
                ]),
            ];
            
            let info = Paragraph::new(info_text)
                .block(Block::default().borders(Borders::ALL).title("Information"));
            
            frame.render_widget(info, chunks[1]);
            
            // Render usage locations
            render_usage_locations(frame, analysis, dep.name.as_str(), chunks[2]);
        }
    } else {
        let loading = Paragraph::new("Loading dependency details...")
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(loading, chunks[1]);
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