use std::collections::HashMap;

use ratatui::layout::{Constraint, Direction, Layout, Rect, Alignment};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, BorderType, List, ListItem, Paragraph, Tabs, Clear, Table, Row, Cell, Gauge, Padding};
use ratatui::Frame;
use unicode_width::UnicodeWidthStr;

use crate::tui::app::App;
use crate::analyzer::{AnalysisResult, DependencyUsage};
use crate::manifest::cargo::DependencyType;

// Modern color palette
pub const PRIMARY_COLOR: Color = Color::Rgb(0, 135, 175);    // Teal
pub const SECONDARY_COLOR: Color = Color::Rgb(0, 175, 135);  // Mint
pub const ACCENT_COLOR: Color = Color::Rgb(175, 135, 0);     // Gold
pub const BG_COLOR: Color = Color::Rgb(30, 30, 46);          // Dark background
pub const TEXT_COLOR: Color = Color::Rgb(220, 220, 230);     // Light text
pub const HIGHLIGHT_COLOR: Color = Color::Rgb(249, 226, 175); // Light yellow
pub const SUCCESS_COLOR: Color = Color::Rgb(87, 187, 138);   // Green
pub const WARNING_COLOR: Color = Color::Rgb(250, 189, 47);   // Yellow
pub const ERROR_COLOR: Color = Color::Rgb(247, 118, 142);    // Red
pub const INACTIVE_COLOR: Color = Color::Rgb(124, 124, 148); // Gray

/// Draw the UI
pub fn draw(frame: &mut Frame, app: &App) {
    // Set default background color
    frame.render_widget(
        Block::default().style(Style::default().bg(BG_COLOR)),
        frame.size()
    );
    
    // Create main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title bar
            Constraint::Length(1),  // Separator
            Constraint::Min(0),     // Main content
            Constraint::Length(2),  // Status bar
        ].as_ref())
        .margin(1)
        .split(frame.size());
    
    // Draw title bar with accent border
    let title = Paragraph::new(Line::from(vec![
        Span::styled(" WHY ", Style::default().bg(ACCENT_COLOR).fg(Color::Black).add_modifier(Modifier::BOLD)),
        Span::raw(" "),
        Span::styled("Dependency Analysis Tool", Style::default().fg(TEXT_COLOR)),
    ]))
    .block(Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(PRIMARY_COLOR))
        .style(Style::default().bg(BG_COLOR)));
    
    frame.render_widget(title, chunks[0]);
    
    // Create tabbed interface
    let titles = vec!["Overview", "Details", "Removable"];
    let tabs = Tabs::new(titles.iter().map(|t| {
        Line::from(vec![
            Span::styled(format!(" {} ", t), Style::default().fg(TEXT_COLOR))
        ])
    }).collect())
    .block(Block::default())
    .select(app.current_tab)
    .style(Style::default().fg(INACTIVE_COLOR))
    .highlight_style(Style::default()
        .fg(HIGHLIGHT_COLOR)
        .add_modifier(Modifier::BOLD));
    
    frame.render_widget(tabs, chunks[0]);
    
    // Draw content based on selected tab
    match app.current_tab {
        0 => draw_overview_tab(frame, app, chunks[2]),
        1 => draw_details_tab(frame, app, chunks[2]),
        2 => draw_removable_tab(frame, app, chunks[2]),
        _ => {}
    }
    
    // Draw status bar
    draw_status_bar(frame, app, chunks[3]);
    
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
    let status_style = Style::default().bg(PRIMARY_COLOR).fg(Color::Black);
    let key_style = Style::default().bg(PRIMARY_COLOR).fg(Color::White).add_modifier(Modifier::BOLD);
    
    let mut status_items = Vec::new();
    
    // Add the sort information
    status_items.push(Span::styled(" Sort:", key_style));
    status_items.push(Span::styled(format!("{} {} ", 
        app.sort_option.as_str(),
        if app.sort_reverse { "↑" } else { "↓" }
    ), status_style));
    
    // Add the filter information
    status_items.push(Span::styled(" Filter:", key_style));
    status_items.push(Span::styled(format!("{} ", app.filter_option.as_str()), status_style));
    
    // Add search query if applicable
    if !app.search_query.is_empty() {
        status_items.push(Span::styled(" Search:", key_style));
        status_items.push(Span::styled(format!("{} ", app.search_query), status_style));
    }
    
    // Fill the remaining space
    let status_text_width: usize = status_items.iter()
        .map(|s| s.content.width())
        .sum();
        
    if area.width as usize > status_text_width {
        status_items.push(Span::styled(
            " ".repeat(area.width as usize - status_text_width),
            status_style
        ));
    }
    
    // Create keybindings help
    let keys = [
        ("q", "Quit"),
        ("Tab", "Switch Tab"),
        ("↑/↓", "Navigate"),
        ("s", "Sort"),
        ("r", "Reverse"),
        ("f", "Filter"),
        ("/", "Search"),
        ("?", "Help"),
    ];
    
    let mut key_spans = Vec::new();
    for (key, desc) in keys {
        key_spans.push(Span::styled(format!(" {} ", key), key_style));
        key_spans.push(Span::styled(format!("{} ", desc), status_style));
    }
    
    // Create final status text
    let top_line = Line::from(status_items);
    let bottom_line = Line::from(key_spans);
    
    let status = Paragraph::new(vec![top_line, bottom_line])
        .block(Block::default())
        .style(status_style);
    
    frame.render_widget(status, area);
}

/// Draw search bar popup
fn draw_search_bar(frame: &mut Frame, app: &App) {
    let area = centered_rect(40, 10, frame.size());
    
    // Clear the area behind the popup
    frame.render_widget(Clear, area);
    
    let search_bar = Paragraph::new(Text::from(format!("{}", app.search_query)))
        .block(Block::default()
            .title(Span::styled(" Search Dependencies ", Style::default().fg(HIGHLIGHT_COLOR)))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(PRIMARY_COLOR))
            .style(Style::default().bg(BG_COLOR).fg(TEXT_COLOR))
            .padding(Padding::horizontal(1)))
        .style(Style::default().fg(TEXT_COLOR));
    
    frame.render_widget(search_bar, area);
}

/// Draw the overview tab
fn draw_overview_tab(frame: &mut Frame, app: &App, area: Rect) {
    // If we have analysis results, delegate to the overview view
    if app.analysis.is_some() {
        crate::tui::views::overview::render(frame, app, area);
    } else {
        // Otherwise show a loading message with a spinner
        let loading_text = format!("Loading analysis... {}", ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]
            [(app.tick_count / 5) % 10]);
        
        let loading = Paragraph::new(loading_text)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(PRIMARY_COLOR))
                .style(Style::default().bg(BG_COLOR)))
            .alignment(Alignment::Center)
            .style(Style::default().fg(TEXT_COLOR));
            
        frame.render_widget(loading, area);
    }
}

/// Draw the details tab
fn draw_details_tab(frame: &mut Frame, app: &App, area: Rect) {
    // If we have analysis results, delegate to the details view
    if app.analysis.is_some() {
        crate::tui::views::details::render(frame, app, area);
    } else {
        // Otherwise show a loading message with a spinner
        let loading_text = format!("Loading analysis... {}", ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]
            [(app.tick_count / 5) % 10]);
        
        let loading = Paragraph::new(loading_text)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(PRIMARY_COLOR))
                .style(Style::default().bg(BG_COLOR)))
            .alignment(Alignment::Center)
            .style(Style::default().fg(TEXT_COLOR));
            
        frame.render_widget(loading, area);
    }
}

/// Draw the removable tab
fn draw_removable_tab(frame: &mut Frame, app: &App, area: Rect) {
    if app.analysis.is_some() {
        // For now, we'll just reuse the overview view with a filter for removable
        // In a full implementation, this would have its own view
        
        let filtered_indices = app.filtered_dependencies()
            .into_iter()
            .filter(|&idx| {
                let dep = &app.analysis.as_ref().unwrap().dependencies[idx];
                app.analysis.as_ref().unwrap().metrics.removable_dependencies.contains(&dep.name)
            })
            .collect::<Vec<_>>();
        
        if filtered_indices.is_empty() {
            let no_removable = Paragraph::new("No removable dependencies found!")
                .block(Block::default()
                    .title(Span::styled(" Removable Dependencies ", Style::default().fg(HIGHLIGHT_COLOR)))
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(PRIMARY_COLOR)))
                .alignment(Alignment::Center)
                .style(Style::default().fg(SUCCESS_COLOR));
                
            frame.render_widget(no_removable, area);
        } else {
            // In a complete implementation, this would show more details about why deps are removable
            crate::tui::views::overview::render(frame, app, area);
        }
    } else {
        // Otherwise show a loading message with a spinner
        let loading_text = format!("Loading analysis... {}", ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]
            [(app.tick_count / 5) % 10]);
        
        let loading = Paragraph::new(loading_text)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(PRIMARY_COLOR))
                .style(Style::default().bg(BG_COLOR)))
            .alignment(Alignment::Center)
            .style(Style::default().fg(TEXT_COLOR));
            
        frame.render_widget(loading, area);
    }
}

/// Draw help popup
fn draw_help(frame: &mut Frame) {
    let area = centered_rect(50, 60, frame.size());
    
    // Clear the area behind the popup
    frame.render_widget(Clear, area);
    
    let help_text = vec![
        Line::from(vec![
            Span::styled("Keyboard Shortcuts", Style::default().fg(HIGHLIGHT_COLOR).add_modifier(Modifier::BOLD))
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Navigation", Style::default().fg(SECONDARY_COLOR).add_modifier(Modifier::BOLD))
        ]),
        Line::from(vec![
            Span::styled("  q, Esc", Style::default().fg(ACCENT_COLOR).add_modifier(Modifier::BOLD)),
            Span::raw(" - Quit application")
        ]),
        Line::from(vec![
            Span::styled("  Tab", Style::default().fg(ACCENT_COLOR).add_modifier(Modifier::BOLD)),
            Span::raw(" - Next tab")
        ]),
        Line::from(vec![
            Span::styled("  Shift+Tab", Style::default().fg(ACCENT_COLOR).add_modifier(Modifier::BOLD)),
            Span::raw(" - Previous tab")
        ]),
        Line::from(vec![
            Span::styled("  ↑/↓, j/k", Style::default().fg(ACCENT_COLOR).add_modifier(Modifier::BOLD)),
            Span::raw(" - Navigate dependencies")
        ]),
        Line::from(vec![
            Span::styled("  ←/→, h/l", Style::default().fg(ACCENT_COLOR).add_modifier(Modifier::BOLD)),
            Span::raw(" - Navigate views in details tab")
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Actions", Style::default().fg(SECONDARY_COLOR).add_modifier(Modifier::BOLD))
        ]),
        Line::from(vec![
            Span::styled("  s", Style::default().fg(ACCENT_COLOR).add_modifier(Modifier::BOLD)),
            Span::raw(" - Cycle sort options")
        ]),
        Line::from(vec![
            Span::styled("  r", Style::default().fg(ACCENT_COLOR).add_modifier(Modifier::BOLD)),
            Span::raw(" - Reverse sort order")
        ]),
        Line::from(vec![
            Span::styled("  f", Style::default().fg(ACCENT_COLOR).add_modifier(Modifier::BOLD)),
            Span::raw(" - Cycle filter options")
        ]),
        Line::from(vec![
            Span::styled("  /", Style::default().fg(ACCENT_COLOR).add_modifier(Modifier::BOLD)),
            Span::raw(" - Search dependencies")
        ]),
        Line::from(vec![
            Span::styled("  Enter", Style::default().fg(ACCENT_COLOR).add_modifier(Modifier::BOLD)),
            Span::raw(" - View dependency details")
        ]),
        Line::from(vec![
            Span::styled("  ?", Style::default().fg(ACCENT_COLOR).add_modifier(Modifier::BOLD)),
            Span::raw(" - Toggle help")
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("Press "),
            Span::styled("Esc", Style::default().fg(ACCENT_COLOR).add_modifier(Modifier::BOLD)),
            Span::raw(" to close this help screen")
        ]),
    ];
    
    let help = Paragraph::new(help_text)
        .block(Block::default()
            .title(Span::styled(" Help ", Style::default().fg(HIGHLIGHT_COLOR)))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(PRIMARY_COLOR))
            .style(Style::default().bg(BG_COLOR)))
        .style(Style::default().fg(TEXT_COLOR));
    
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

/// Calculate a color based on importance score
pub fn importance_color(score: f64) -> Color {
    if score > 0.7 {
        SUCCESS_COLOR
    } else if score > 0.3 {
        WARNING_COLOR
    } else {
        ERROR_COLOR
    }
}

// Helper struct for margins
pub struct Margin {
    pub vertical: u16,
    pub horizontal: u16,
}

// Extension trait to apply margins to a Rect
pub trait RectExt {
    fn inner(&self, margin: &Margin) -> Rect;
}

impl RectExt for Rect {
    fn inner(&self, margin: &Margin) -> Rect {
        let horizontal_margin = margin.horizontal.min(self.width / 2);
        let vertical_margin = margin.vertical.min(self.height / 2);
        
        Rect {
            x: self.x + horizontal_margin,
            y: self.y + vertical_margin,
            width: self.width - horizontal_margin * 2,
            height: self.height - vertical_margin * 2,
        }
    }
} 