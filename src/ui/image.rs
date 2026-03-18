use crate::app::App;
use crate::ui::{C_ACCENT, C_DIM};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
    Frame,
};

pub fn draw_cover(f: &mut Frame, app: &App, area: Rect) {
    if let Some(bytes) = &app.cover_image {
        if let Ok(img) = image::load_from_memory(bytes) {
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Rgb(28, 28, 28)))
                .style(Style::default().bg(Color::Rgb(0, 0, 0)));
            let inner = block.inner(area);
            f.render_widget(block, area);

            // If we have a native protocol, we can use ratatui-image
            // First we need to convert to dynamic image
            let dyn_img = image::DynamicImage::ImageRgba8(img.into_rgba8());
            
            // Create a picker
            let mut picker = ratatui_image::picker::Picker::from_query_stdio()
                .unwrap_or_else(|_| ratatui_image::picker::Picker::from_fontsize((8, 16)));
            
            // Create the protocol
            let mut protocol = picker.new_resize_protocol(dyn_img);
            let image_widget = ratatui_image::StatefulImage::default()
                .resize(ratatui_image::Resize::Fit(Some(image::imageops::FilterType::Lanczos3)));
            f.render_stateful_widget(image_widget, inner, &mut protocol);
            return;
        }
    }
    let loading = app.is_searching || (app.selected.is_some() && app.cover_image.is_none());
    f.render_widget(
        Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled("  ┌──────┐", Style::default().fg(Color::Rgb(35,35,35)))),
            Line::from(Span::styled("  │      │", Style::default().fg(Color::Rgb(35,35,35)))),
            Line::from(Span::styled("  │  ◆   │", Style::default().fg(C_ACCENT))),
            Line::from(Span::styled("  │      │", Style::default().fg(Color::Rgb(35,35,35)))),
            Line::from(Span::styled("  └──────┘", Style::default().fg(Color::Rgb(35,35,35)))),
            Line::from(""),
            Line::from(Span::styled(
                if loading { "  loading…" } else { "  no cover" },
                Style::default().fg(C_DIM),
            )),
        ])
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(22,22,22)))
            .style(Style::default().bg(Color::Rgb(0,0,0)))),
        area,
    );
}

