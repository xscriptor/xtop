mod app;
mod theme;
mod tui;
mod ui;

use std::{error::Error, io, time::Duration};
use crossterm::event::{self, Event, KeyCode};
use app::App;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tui::install_panic_hook();
    let mut terminal = tui::init()?;
    let mut app = App::new();
    let tick_rate = Duration::from_millis(1000); // 1 second update
    let mut last_tick = std::time::Instant::now();

    loop {
        terminal.draw(|f| ui::ui(f, &app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => app.quit(),
                    KeyCode::Char('t') => app.next_theme(),
                    KeyCode::Char('T') => app.previous_theme(),
                    KeyCode::Char('l') => app.next_layout(),
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = std::time::Instant::now();
        }

        if app.should_quit {
            break;
        }
    }

    tui::restore()?;
    Ok(())
}
