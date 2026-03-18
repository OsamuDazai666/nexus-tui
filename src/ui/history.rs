use crate::app::{App, Focus};
use crate::ui::{focused_block, trunc, C_ACCENT,
                C_BORDER_F, C_DIM, C_GREEN, C_PANEL, C_TEXT};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, ListState, Paragraph},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let cols = Layout::horizontal([
        Constraint::Percentage(36),
        Constraint::Percentage(64),
    ]).split(area);
    draw_list(f, app, cols[0]);
    draw_entry(f, app, cols[1]);
}

fn draw_list(f: &mut Frame, app: &App, area: Rect) {
    let focused = app.focus == Focus::History;

    let items: Vec<ListItem> = app.history.iter().enumerate().map(|(i, e)| {
        let sel  = i == app.history_idx;
        let date = e.last_watched.format("%b %d  %H:%M").to_string();
        let bar  = e.progress_bar(10);
        let pct  = e.progress_pct()
            .map(|p| format!(" {:.0}%", p * 100.0))
            .unwrap_or_default();

        if sel {
            ListItem::new(vec![
                Line::from(Span::styled(
                    format!(" ▶ {:─<24}", trunc(&e.title, 23)),
                    Style::default().fg(Color::Rgb(0,0,0)).bg(C_ACCENT).add_modifier(Modifier::BOLD),
                )),
                Line::from(Span::styled(
                    format!("   {}  {}{} ", e.media_type.to_uppercase(), bar, pct),
                    Style::default().fg(Color::Rgb(60,60,0)).bg(C_ACCENT),
                )),
            ])
        } else {
            ListItem::new(vec![
                Line::from(vec![
                    Span::styled("  ", Style::default().bg(C_PANEL)),
                    Span::styled(trunc(&e.title, 24), Style::default().fg(C_TEXT).bg(C_PANEL)),
                ]),
                Line::from(vec![
                    Span::styled(format!("  {}  {}", date, pct), Style::default().fg(C_DIM).bg(C_PANEL)),
                ]),
            ])
        }
    }).collect();

    let title = match app.history.len() {
        0 => " HISTORY ".to_string(),
        n => format!(" HISTORY  {n} "),
    };

    let list = List::new(items)
        .block(Block::default()
            .title(Span::styled(title, Style::default()
                .fg(if focused { C_ACCENT } else { C_DIM })
                .add_modifier(Modifier::BOLD)))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(if focused { C_BORDER_F } else { Color::Rgb(28,28,28) }))
            .style(Style::default().bg(C_PANEL)));

    let mut state = ListState::default();
    if !app.history.is_empty() { state.select(Some(app.history_idx)); }
    f.render_stateful_widget(list, area, &mut state);
}

fn draw_entry(f: &mut Frame, app: &App, area: Rect) {
    let rows = Layout::vertical([
        Constraint::Min(0),
        Constraint::Length(5),
    ]).split(area);

    if let Some(e) = app.history.get(app.history_idx) {
        let mut lines = vec![
            Line::from(""),
            Line::from(Span::styled(
                &e.title,
                Style::default().fg(C_TEXT).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("  TYPE    ", Style::default().fg(C_DIM)),
                Span::styled(&e.media_type, Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("  PLAYED  ", Style::default().fg(C_DIM)),
                Span::styled(format!("{}×", e.play_count), Style::default().fg(C_TEXT).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("  LAST    ", Style::default().fg(C_DIM)),
                Span::styled(
                    e.last_watched.format("%Y-%m-%d  %H:%M").to_string(),
                    Style::default().fg(C_TEXT),
                ),
            ]),
        ];

        if let (Some(p), Some(t)) = (e.progress, e.total) {
            lines.push(Line::from(vec![
                Span::styled("  PROG    ", Style::default().fg(C_DIM)),
                Span::styled(
                    format!("{p} / {t}"),
                    Style::default().fg(C_GREEN).add_modifier(Modifier::BOLD),
                ),
            ]));
        }

        lines.push(Line::from(""));
        if e.stream_url.is_some() {
            lines.push(Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(" [P] REWATCH ", Style::default()
                    .fg(Color::Rgb(0,0,0)).bg(C_ACCENT).add_modifier(Modifier::BOLD)),
                Span::styled("   ", Style::default()),
                Span::styled("[D] REMOVE", Style::default().fg(C_DIM)),
            ]));
        } else {
            lines.push(Line::from(
                Span::styled("  [D] remove from history", Style::default().fg(C_DIM)),
            ));
        }

        f.render_widget(
            Paragraph::new(lines)
                .block(focused_block(" ENTRY ", false)),
            rows[0],
        );

        // Progress gauge — yellow fill
        let pct   = e.progress_pct().unwrap_or(0.0);
        let label = if let (Some(p), Some(t)) = (e.progress, e.total) {
            format!("{p} / {t}  {:.0}%", pct * 100.0)
        } else {
            "no progress tracked".to_string()
        };

        f.render_widget(
            Gauge::default()
                .block(Block::default()
                    .title(Span::styled(" PROGRESS ", Style::default().fg(C_DIM)))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Rgb(28,28,28)))
                    .style(Style::default().bg(C_PANEL)))
                .gauge_style(Style::default().fg(C_ACCENT).bg(Color::Rgb(20,20,20)))
                .ratio(pct)
                .label(Span::styled(label, Style::default().fg(C_TEXT).add_modifier(Modifier::BOLD))),
            rows[1],
        );
    } else {
        f.render_widget(
            Paragraph::new(vec![
                Line::from(""),
                Line::from(Span::styled("  Nothing watched yet.", Style::default().fg(C_DIM))),
                Line::from(""),
                Line::from(Span::styled("  Search something and press [P] to play.", Style::default().fg(C_DIM))),
            ])
            .block(focused_block(" HISTORY ", false)),
            area,
        );
    }
}
