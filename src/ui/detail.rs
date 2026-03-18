use crate::app::{App, Focus};
use crate::ui::{focused_block, trunc, C_ACCENT, C_DIM,
                C_GREEN, C_PANEL, C_RED, C_SCORE, C_TEXT};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

// ── Meta (title, score, genres, actions) ─────────────────────────────────────

pub fn draw_meta(f: &mut Frame, app: &App, area: Rect) {
    let focused = app.focus == Focus::Detail;

    let Some(item) = &app.selected else {
        f.render_widget(
            Paragraph::new(vec![
                Line::from(""),
                Line::from(Span::styled("  ← select a result", Style::default().fg(C_DIM))),
            ])
            .block(focused_block(" INFO ", focused)),
            area,
        );
        return;
    };

    let mut lines: Vec<Line> = vec![Line::from("")];

    // Title — bold white, all caps feel
    lines.push(Line::from(Span::styled(
        item.title(),
        Style::default().fg(C_TEXT).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    // Score
    if let Some(s) = item.score() {
        let color = if s >= 8.5 { C_GREEN } else if s >= 7.0 { C_SCORE } else { C_DIM };
        lines.push(Line::from(vec![
            Span::styled("  ★ ", Style::default().fg(color)),
            Span::styled(
                format!("{s:.1}  {}", stars(s)),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
        ]));
    }

    // Year · episodes · status
    {
        let mut row = vec![Span::styled("  ", Style::default())];
        if let Some(y) = item.year() {
            row.push(Span::styled(y.to_string(), Style::default().fg(C_DIM)));
        }
        if let Some(ep) = item.episodes_or_chapters() {
            row.push(Span::styled("  ·  ", Style::default().fg(Color::Rgb(40,40,40))));
            row.push(Span::styled(ep, Style::default().fg(C_DIM)));
        }
        lines.push(Line::from(row));
    }

    if let Some(status) = item.status() {
        let (dot, col) = status_dot(status);
        lines.push(Line::from(vec![
            Span::styled(format!("  {dot} "), Style::default().fg(col)),
            Span::styled(status, Style::default().fg(col)),
        ]));
    }

    lines.push(Line::from(""));

    // Genres — white on dark-grey chips
    let genres = item.genres();
    if !genres.is_empty() {
        let mut row = vec![Span::styled(" ", Style::default())];
        for g in genres.iter().take(4) {
            row.push(Span::styled(
                format!(" {g} "),
                Style::default()
                    .fg(C_TEXT)
                    .bg(Color::Rgb(30, 30, 30))
                    .add_modifier(Modifier::BOLD),
            ));
            row.push(Span::styled(" ", Style::default()));
        }
        lines.push(Line::from(row));
        lines.push(Line::from(""));
    }

    // Source badge
    lines.push(Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(
            format!(" {} ", item.source_badge()),
            Style::default().fg(C_DIM).bg(Color::Rgb(22,22,22)),
        ),
    ]));

    lines.push(Line::from(""));

    // Action hints — yellow accent keys
    lines.push(Line::from(vec![
        Span::styled("  ", Style::default()),
        key_chip("[P]"), Span::styled(" Play  ", Style::default().fg(C_DIM)),
        key_chip("[R]"), Span::styled(" Related", Style::default().fg(C_DIM)),
    ]));

    f.render_widget(
        Paragraph::new(lines)
            .block(focused_block(" INFO ", focused))
            .wrap(Wrap { trim: true }),
        area,
    );
}

// ── Synopsis ──────────────────────────────────────────────────────────────────

pub fn draw_synopsis(f: &mut Frame, app: &App, area: Rect) {
    let focused = app.focus == Focus::Detail;
    let text = app.selected.as_ref()
        .map(|i| i.synopsis())
        .unwrap_or("No synopsis.");

    f.render_widget(
        Paragraph::new(text)
            .style(Style::default().fg(Color::Rgb(180, 180, 180)))
            .block(focused_block(" SYNOPSIS ", focused))
            .wrap(Wrap { trim: true })
            .scroll((app.detail_scroll, 0)),
        area,
    );
}

// ── Recommendations ───────────────────────────────────────────────────────────

pub fn draw_recommendations(f: &mut Frame, app: &App, area: Rect) {
    let focused = app.focus == Focus::Recommendations;

    let items: Vec<ListItem> = app.recommendations.iter().enumerate().map(|(i, rec)| {
        let sel   = i == app.recs_idx;
        let score = rec.score().map(|s| format!("★{:.1}", s)).unwrap_or_default();

        if sel {
            ListItem::new(Line::from(Span::styled(
                format!(" ▶ {}  {} ", trunc(rec.title(), 26), score),
                Style::default().fg(Color::Rgb(0,0,0)).bg(C_ACCENT).add_modifier(Modifier::BOLD),
            )))
        } else {
            ListItem::new(Line::from(vec![
                Span::styled("   ", Style::default().bg(C_PANEL)),
                Span::styled(trunc(rec.title(), 26), Style::default().fg(C_TEXT).bg(C_PANEL)),
                Span::styled(format!("  {score}"), Style::default().fg(C_SCORE).bg(C_PANEL)),
            ]))
        }
    }).collect();

    let title = if app.recommendations.is_empty() {
        " RELATED ".to_string()
    } else {
        format!(" RELATED  {} ", app.recommendations.len())
    };

    let list = List::new(items)
        .block(Block::default()
            .title(Span::styled(title, Style::default()
                .fg(if focused { C_ACCENT } else { C_DIM })
                .add_modifier(Modifier::BOLD)))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(if focused {
                crate::ui::C_BORDER_F
            } else {
                Color::Rgb(28,28,28)
            }))
            .style(Style::default().bg(C_PANEL)));

    let mut state = ListState::default();
    if focused && !app.recommendations.is_empty() { state.select(Some(app.recs_idx)); }
    f.render_stateful_widget(list, area, &mut state);
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn stars(s: f32) -> &'static str {
    match s as u32 { 9..=10=>"★★★★★", 8=>"★★★★☆", 7=>"★★★☆☆", 6=>"★★☆☆☆", _=>"★☆☆☆☆" }
}

fn status_dot(s: &str) -> (&'static str, Color) {
    let l = s.to_lowercase();
    if l.contains("finish") || l.contains("complet") || l.contains("released") { ("●", C_GREEN) }
    else if l.contains("airing") || l.contains("ongoing") || l.contains("returning") { ("●", C_ACCENT) }
    else if l.contains("cancel") { ("●", C_RED) }
    else { ("○", C_DIM) }
}

fn key_chip(s: &'static str) -> Span<'static> {
    Span::styled(s, Style::default()
        .fg(Color::Rgb(0,0,0))
        .bg(C_ACCENT)
        .add_modifier(Modifier::BOLD))
}
