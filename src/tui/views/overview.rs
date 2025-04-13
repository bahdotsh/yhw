use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

use crate::analyzer::AnalysisResult;
use crate::tui::app::App;

/// Render the overview view
pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    // Create layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
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
        render_dependencies_list(frame, app, analysis, chunks[1]);
    } else {
        let loading = Paragraph::new("Loading dependencies...").block(Block::default().borders(Borders::ALL));
        frame.render_widget(loading, chunks[1]);
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
            
            ListItem::new(Line::from(vec![
                Span::styled(&dep.name, name_style),
                Span::raw(" "),
                Span::styled(format!("({:.2})", importance), Style::default().fg(Color::Gray))
            ])).style(style)
        })
        .collect();
    
    let deps_list = List::new(deps)
        .block(Block::default().borders(Borders::ALL).title("Dependencies"))
        .highlight_style(Style::default().bg(Color::DarkGray))
        .highlight_symbol("> ");
    
    // Create a mutable list state based on selected dependency
    let mut list_state = ratatui::widgets::ListState::default();
    list_state.select(Some(app.selected_dependency));
    
    frame.render_stateful_widget(deps_list, area, &mut list_state);
} 