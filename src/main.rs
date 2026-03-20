mod api;
mod app;
mod config;
mod db;
mod player;
mod ui;

use anyhow::Result;
use app::App;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, time::Duration};

#[tokio::main]
async fn main() -> Result<()> {
    // Write sample config if first run
    let _ = config::Config::write_sample();

    // Enable raw mode BEFORE the stdio query so Windows' console echo is
    // suppressed. On Linux/macOS order doesn't matter, but on Windows the
    // picker query writes escape sequences to stdout and reads responses from
    // stdin — if echo is still on, those bytes bleed back as keystrokes after
    // startup, causing every key to register twice.
    enable_raw_mode()?;
    let mut stdout = io::stdout();

    // Probe terminal for graphics protocol support (Kitty/Sixel/iTerm2).
    // Must happen before EnterAlternateScreen so the query/response round-trip
    // goes to the main screen buffer, not the alternate screen.
    let mut image_picker = ratatui_image::picker::Picker::from_query_stdio()
        .unwrap_or_else(|_| ratatui_image::picker::Picker::from_fontsize((8, 16)));

    // The library's iterm2_from_env() maps "vscode" → Iterm2, but VS Code's
    // terminal (xterm.js) actually supports Sixel, not iTerm2 inline images
    // (OSC 1337). Override to Sixel for VS Code so we get proper rendering.
    let term_prog = std::env::var("TERM_PROGRAM").unwrap_or_default();
    if term_prog.contains("vscode") {
        image_picker.set_protocol_type(ratatui_image::picker::ProtocolType::Sixel);
    } else if image_picker.protocol_type() == ratatui_image::picker::ProtocolType::Halfblocks {
        // Safety net for other terminals: override Halfblocks → Iterm2 where supported.
        if term_prog.contains("iTerm")
            || term_prog.contains("WezTerm")
            || term_prog.contains("mintty")
            || term_prog.contains("Tabby")
            || term_prog.contains("Hyper")
            || term_prog.contains("rio")
        {
            image_picker.set_protocol_type(ratatui_image::picker::ProtocolType::Iterm2);
        }
    }

    // Enter alternate screen and set up terminal
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(image_picker).await?;
    let result  = run(&mut terminal, &mut app).await;

    // Always restore terminal, even on error
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    if let Err(ref e) = result {
        eprintln!("\nnexus-tui error: {e}");
    }

    result
}

async fn run<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    loop {
        if app.needs_redraw {
            terminal.clear()?;
            app.needs_redraw = false;
        }
        terminal.draw(|f| ui::draw(f, app))?;

        // Poll for input (non-blocking — 100ms max)
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    if app.handle_key(key).await? {
                        return Ok(());
                    }
                }
                Event::Resize(_, _) => {
                    app.on_resize();
                }
                _ => {}
            }
        }

        // Background task tick (debounce, spinner, message drain)
        app.tick().await?;
    }
}