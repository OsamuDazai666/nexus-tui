use crate::app::{App, Focus};
use crate::ui::{focused_block, trunc, C_ACCENT, C_BG, C_BG2, C_BG3,
                C_BORDER_F, C_CYAN, C_DIM, C_GREEN, C_PANEL, C_SCORE, C_TEXT};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

// ── Search bar ────────────────────────────────────────────────────────────────

pub fn draw_search_bar(f: &mut Frame, app: &App, area: Rect) {
    let focused = app.focus == Focus::Search;

    let content = if app.search_input.is_empty() && !focused {
        Line::from(Span::styled("  search…", Style::default().fg(C_DIM)))
    } else {
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(&app.search_input, Style::default().fg(C_TEXT).add_modifier(Modifier::BOLD)),
            Span::styled(
                if focused { "▌" } else { "" },
                Style::default().fg(C_ACCENT),
            ),
        ])
    };

    f.render_widget(
        Paragraph::new(content)
            .block(Block::default()
                .title(Span::styled(
                    if focused { " SEARCH " } else { " SEARCH [/] " },
                    Style::default()
                        .fg(if focused { C_ACCENT } else { C_DIM })
                        .add_modifier(if focused { Modifier::BOLD } else { Modifier::empty() }),
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(if focused { C_BORDER_F } else { Color::Rgb(28,28,28) }))
                .style(Style::default().bg(if focused { C_BG3 } else { C_BG2 }))),
        area,
    );
}

// ── Results list ──────────────────────────────────────────────────────────────

pub fn draw_results(f: &mut Frame, app: &App, area: Rect) {
    let focused = app.focus == Focus::Results;

    let items: Vec<ListItem> = app.results.iter().enumerate().map(|(i, item)| {
        let sel   = i == app.results_idx;
        let score = item.score().map(|s| format!("{:.1}", s)).unwrap_or_default();
        let year  = item.year().map(|y| y.to_string()).unwrap_or_default();

        let _type_color = match item.media_type() {
            crate::api::MediaType::Anime  => C_ACCENT,
            crate::api::MediaType::Movie  => C_CYAN,
            crate::api::MediaType::TV     => C_GREEN,
            crate::api::MediaType::Manga  => Color::Rgb(200, 100, 255),
        };

        let eps = item.episodes_or_chapters().unwrap_or_default();

        if sel {
            // Selected row — yellow bg, black text
            let title_str = if eps.is_empty() {
                format!(" {:─<25}", trunc(item.title(), 24))
            } else {
                format!(" {:─<25} {} ", trunc(item.title(), 24), eps)
            };
            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(
                        title_str,
                        Style::default().fg(Color::Rgb(0,0,0)).bg(C_ACCENT).add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::styled(
                        format!(" {}  ★{}  {} ", year, score, item.source_badge()),
                        Style::default().fg(Color::Rgb(60,60,0)).bg(C_ACCENT),
                    ),
                ]),
            ])
        } else {
            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(" ", Style::default().bg(C_PANEL)),
                    Span::styled(trunc(item.title(), 24), Style::default().fg(C_TEXT).bg(C_PANEL)),
                    Span::styled(if eps.is_empty() { "".to_string() } else { format!("  {}", eps) }, Style::default().fg(C_DIM).bg(C_PANEL)),
                ]),
                Line::from(vec![
                    Span::styled(format!(" {} ", year), Style::default().fg(C_DIM).bg(C_PANEL)),
                    Span::styled(format!("★{} ", score), Style::default().fg(C_SCORE).bg(C_PANEL)),
                ]),
            ])
        }
    }).collect();

    let title = match (app.results.len(), app.is_searching) {
        (_, true)  => format!(" {} SEARCHING ", app.spinner.symbol()),
        (0, false) => " RESULTS ".to_string(),
        (n, false) => format!(" RESULTS  {n}{} ", if app.has_more { "+" } else { "" }),
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
    if !app.results.is_empty() { state.select(Some(app.results_idx)); }
    f.render_stateful_widget(list, area, &mut state);
}
