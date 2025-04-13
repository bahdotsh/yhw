use ratatui::layout::{Constraint, Direction, Layout, Rect, Alignment};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, BorderType, List, ListItem, Paragraph, Table, Row, Cell, Gauge};
use ratatui::Frame;

use crate::analyzer::AnalysisResult;
use crate::tui::app::App;
use crate::manifest::cargo::DependencyType;
use crate::tui::ui::{PRIMARY_COLOR, SECONDARY_COLOR, ACCENT_COLOR, BG_COLOR, TEXT_COLOR, 
                  HIGHLIGHT_COLOR, SUCCESS_COLOR, WARNING_COLOR, ERROR_COLOR, INACTIVE_COLOR};

/// Render the overview view
pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    // Create layout for the overview
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),  // Summary stats
            Constraint::Min(0),     // Dependency list
        ].as_ref())
        .split(area);
    
    if let Some(analysis) = &app.analysis {
        // Render dependency summary with visualizations
        render_dependency_summary(frame, analysis, chunks[0]);
        
        // Render the dependency list with modern styling
        render_dependencies_list(frame, app, analysis, chunks[1]);
    } else {
        let loading_text = format!("Loading dependencies... {}", ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]
            [(app.tick_count / 5) % 10]);
            
        let loading = Paragraph::new(loading_text)
            .block(Block::default()
                .title(Span::styled(" Overview ", Style::default().fg(HIGHLIGHT_COLOR)))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(PRIMARY_COLOR)))
            .alignment(Alignment::Center)
            .style(Style::default().fg(TEXT_COLOR));
            
        frame.render_widget(loading, area);
    }
}

/// Render a summary of the dependency analysis with modern visualizations
fn render_dependency_summary(frame: &mut Frame, analysis: &AnalysisResult, area: Rect) {
    // Split the summary area into two columns
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60),  // Stats
            Constraint::Percentage(40),  // Donut chart
        ].as_ref())
        .split(area);
    
    // Calculate summary statistics
    let total_deps = analysis.dependencies.len();
    let normal_deps = analysis.dependencies.iter()
        .filter(|d| matches!(d.dependency_type, DependencyType::Normal))
        .count();
    let dev_deps = analysis.dependencies.iter()
        .filter(|d| matches!(d.dependency_type, DependencyType::Development))
        .count();
    let build_deps = analysis.dependencies.iter()
        .filter(|d| matches!(d.dependency_type, DependencyType::Build))
        .count();
    
    let unused_deps = analysis.metrics.is_used.iter()
        .filter(|(_, &is_used)| !is_used)
        .count();
    let removable_deps = analysis.metrics.removable_dependencies.len();
    
    // Create gauges for different metrics
    let normal_gauge = Gauge::default()
        .block(Block::default().borders(Borders::NONE))
        .gauge_style(Style::default().fg(PRIMARY_COLOR).bg(BG_COLOR))
        .ratio((normal_deps as f64) / (total_deps as f64))
        .label(format!("Normal: {}/{} ({}%)", 
            normal_deps, 
            total_deps,
            (normal_deps as f64 * 100.0 / total_deps as f64) as u32
        ));
    
    let dev_gauge = Gauge::default()
        .block(Block::default().borders(Borders::NONE))
        .gauge_style(Style::default().fg(SECONDARY_COLOR).bg(BG_COLOR))
        .ratio((dev_deps as f64) / (total_deps as f64))
        .label(format!("Dev: {}/{} ({}%)", 
            dev_deps, 
            total_deps,
            (dev_deps as f64 * 100.0 / total_deps as f64) as u32
        ));
    
    let build_gauge = Gauge::default()
        .block(Block::default().borders(Borders::NONE))
        .gauge_style(Style::default().fg(ACCENT_COLOR).bg(BG_COLOR))
        .ratio((build_deps as f64) / (total_deps as f64))
        .label(format!("Build: {}/{} ({}%)", 
            build_deps, 
            total_deps,
            (build_deps as f64 * 100.0 / total_deps as f64) as u32
        ));
    
    let removable_gauge = Gauge::default()
        .block(Block::default().borders(Borders::NONE))
        .gauge_style(Style::default().fg(ERROR_COLOR).bg(BG_COLOR))
        .ratio((removable_deps as f64) / (total_deps as f64))
        .label(format!("Removable: {}/{} ({}%)", 
            removable_deps, 
            total_deps,
            (removable_deps as f64 * 100.0 / total_deps as f64) as u32
        ));
    
    // Create a block for the summary section
    let summary_block = Block::default()
        .title(Span::styled(" Dependency Summary ", Style::default().fg(HIGHLIGHT_COLOR)))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(PRIMARY_COLOR));
    
    frame.render_widget(summary_block, area);
    
    // Layout for the gauges using standard methods
    let gauge_area = Rect {
        x: chunks[0].x + 2,
        y: chunks[0].y + 1,
        width: chunks[0].width - 4,
        height: chunks[0].height - 2,
    };
    
    let gauge_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ].as_ref())
        .split(gauge_area);
    
    // Render the gauges
    frame.render_widget(normal_gauge, gauge_chunks[0]);
    frame.render_widget(dev_gauge, gauge_chunks[1]);
    frame.render_widget(build_gauge, gauge_chunks[2]);
    frame.render_widget(removable_gauge, gauge_chunks[3]);
    
    // Create bars for the chart in the right column
    let chart_data = [
        ("Normal", normal_deps, PRIMARY_COLOR),
        ("Dev", dev_deps, SECONDARY_COLOR),
        ("Build", build_deps, ACCENT_COLOR),
        ("Removable", removable_deps, ERROR_COLOR),
        ("Unused", unused_deps, INACTIVE_COLOR),
    ];
    
    let max_value = chart_data.iter().map(|(_, count, _)| *count).max().unwrap_or(1);
    
    // Create a bar chart in the right column using standard methods
    let chart_area = Rect {
        x: chunks[1].x + 2,
        y: chunks[1].y + 2,
        width: chunks[1].width - 4,
        height: chunks[1].height - 4,
    };
    
    // Create bars with styled labels
    let bars: Vec<(String, u64)> = chart_data
        .iter()
        .map(|(name, count, _)| (name.to_string(), *count as u64))
        .collect();
    
    // Create custom styled bars
    let mut styled_rows = Vec::new();
    for (_, (name, count, color)) in chart_data.iter().enumerate() {
        let bar_width = ((*count as f64 / max_value as f64) * (chart_area.width as f64 - 15.0)) as u16;
        let bar = "█".repeat(bar_width as usize);
        
        styled_rows.push(Row::new(vec![
            Cell::from(name.to_string()).style(Style::default().fg(TEXT_COLOR)),
            Cell::from(count.to_string()).style(Style::default().fg(TEXT_COLOR)),
            Cell::from(bar).style(Style::default().fg(*color)),
        ]));
    }
    
    // Create a table to display the bars
    let table = Table::new(styled_rows)
        .block(Block::default().borders(Borders::NONE))
        .style(Style::default().fg(TEXT_COLOR))
        .widths(&[
            Constraint::Length(10),
            Constraint::Length(5),
            Constraint::Min(0),
        ]);
    
    frame.render_widget(table, chart_area);
}

/// Render the dependency list with enhanced styling
fn render_dependencies_list(frame: &mut Frame, app: &App, analysis: &AnalysisResult, area: Rect) {
    let filtered_indices = app.filtered_dependencies();
    
    // Create styled list items
    let deps: Vec<ListItem> = filtered_indices
        .iter()
        .enumerate()
        .map(|(list_idx, &dep_idx)| {
            let dep = &analysis.dependencies[dep_idx];
            
            let _is_selected = list_idx == app.selected_dependency;
            
            let used = analysis.metrics.is_used.get(&dep.name).unwrap_or(&false);
            let importance = analysis.metrics.importance_scores.get(&dep.name).unwrap_or(&0.0);
            let is_removable = analysis.metrics.removable_dependencies.contains(&dep.name);
            let usage_count = analysis.metrics.usage_count.get(&dep.name).unwrap_or(&0);
            
            // Show dependency name with color based on importance
            let name_style = if *used {
                if *importance > 0.7 {
                    Style::default().fg(SUCCESS_COLOR)
                } else if *importance > 0.3 {
                    Style::default().fg(WARNING_COLOR)
                } else {
                    Style::default().fg(ERROR_COLOR)
                }
            } else {
                Style::default().fg(INACTIVE_COLOR)
            };
            
            // Show type indicator with icon
            let type_icon = match dep.dependency_type {
                DependencyType::Normal => "📦",
                DependencyType::Development => "🔧",
                DependencyType::Build => "🏗️",
            };
            
            // Show removable indicator
            let removable_icon = if is_removable { "🗑️" } else { "" };
            
            // Create mini usage graph using unicode block characters
            let max_graph_width = 10;
            let graph_width = ((usage_count * max_graph_width) / 
                analysis.metrics.usage_count.values().map(|v| *v).max().unwrap_or(1)).min(max_graph_width);
            let usage_graph = "█".repeat(graph_width);
            let empty_graph = "░".repeat(max_graph_width - graph_width);
            
            let graph_style = if *importance > 0.7 {
                Style::default().fg(SUCCESS_COLOR)
            } else if *importance > 0.3 {
                Style::default().fg(WARNING_COLOR)
            } else {
                Style::default().fg(ERROR_COLOR)
            };
            
            // Format version info
            let version = dep.version.as_deref().unwrap_or("unknown");
            
            // Create a line with all this information
            ListItem::new(Line::from(vec![
                Span::raw(format!("{} ", type_icon)),
                Span::styled(&dep.name, name_style),
                Span::raw(" "),
                Span::styled(format!("({})", version), Style::default().fg(INACTIVE_COLOR)),
                Span::raw("  "),
                Span::styled(usage_graph, graph_style),
                Span::styled(empty_graph, Style::default().fg(INACTIVE_COLOR)),
                Span::raw(format!(" {}", usage_count)),
                Span::raw("  "),
                Span::styled(removable_icon, Style::default().fg(ERROR_COLOR)),
            ]))
        })
        .collect();
    
    // Create the list with enhanced styling
    let list_title = if filtered_indices.len() != analysis.dependencies.len() {
        format!(" Dependencies ({} of {} shown) ", filtered_indices.len(), analysis.dependencies.len())
    } else {
        " Dependencies ".to_string()
    };
    
    let list = List::new(deps)
        .block(Block::default()
            .title(Span::styled(list_title, Style::default().fg(HIGHLIGHT_COLOR)))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(PRIMARY_COLOR)))
        .style(Style::default().fg(TEXT_COLOR))
        .highlight_style(
            Style::default()
                .bg(PRIMARY_COLOR)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD)
        )
        .highlight_symbol("▶ ");
    
    frame.render_widget(list, area);
} 