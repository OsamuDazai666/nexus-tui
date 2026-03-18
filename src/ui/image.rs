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

            let pw = (inner.width as u32).max(1);
            let ph = (inner.height as u32 * 2).max(1);
            let resized = img.resize_exact(pw, ph, image::imageops::FilterType::Triangle);
            let rgba    = resized.to_rgba8();
            f.render_widget(HalfBlock { rgba }, inner);
            return;
        }
    }

    // Placeholder
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

struct HalfBlock { rgba: image::RgbaImage }

impl Widget for HalfBlock {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let (iw, ih) = self.rgba.dimensions();
        for row in 0..area.height {
            for col in 0..area.width {
                let px      = col as u32;
                let py_top  = (row * 2) as u32;
                let py_bot  = py_top + 1;

                let top = pixel_color(&self.rgba, px, py_top, iw, ih);
                let bot = pixel_color(&self.rgba, px, py_bot, iw, ih);

                if let Some(cell) = buf.cell_mut((area.x + col, area.y + row)) {
                    cell.set_char('▀');
                    cell.set_fg(top);
                    cell.set_bg(bot);
                }
            }
        }
    }
}

fn pixel_color(img: &image::RgbaImage, x: u32, y: u32, w: u32, h: u32) -> Color {
    if x < w && y < h {
        let p = img.get_pixel(x, y);
        Color::Rgb(p[0], p[1], p[2])
    } else {
        Color::Rgb(0, 0, 0)
    }
}
