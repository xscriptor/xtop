use crossterm::event::{self, Event, KeyCode};
use std::time::{Duration, Instant};
use xtop_core::application::state::{AppState, Config, InputMode};
use xtop_core::infrastructure::config;
use xtop_core::infrastructure::sysinfo_provider::SysinfoProvider;
use xtop_core::infrastructure::theme_loader::load_all_themes;
use xtop_tui::render;
use xtop_tui::terminal;

fn main() -> anyhow::Result<()> {
    terminal::install_panic_hook();
    let mut terminal = terminal::init()?;

    let provider = SysinfoProvider::new();
    let themes = load_all_themes();
    let cfg = config::load_config();
    let mut state = AppState::new(Box::new(provider), themes, cfg);

    let tick_rate = Duration::from_millis(state.update_interval_ms);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| render::render(f, &state))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_default();

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match state.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('q') => {
                            let cfg = Config {
                                theme: state.current_theme.name.clone(),
                                layout_mode: state.layout_mode,
                                update_interval_ms: state.update_interval_ms,
                                history_points: 100,
                                alerts: state.alerts,
                            };
                            let _ = config::save_config(&cfg);
                            state.quit();
                        }
                        KeyCode::Char('?') => state.toggle_help(),
                        KeyCode::Char('t') => state.next_theme(),
                        KeyCode::Char('T') => state.previous_theme(),
                        KeyCode::Char('l') => state.next_layout(),
                        KeyCode::Char('f') => state.toggle_fullscreen(),
                        KeyCode::Char('F') => state.cycle_fullscreen_widget(),
                        KeyCode::Char('/') => state.start_search(),
                        KeyCode::Esc => {
                            if state.show_help {
                                state.toggle_help();
                            }
                        }
                        _ => {}
                    },
                    InputMode::Searching => match key.code {
                        KeyCode::Esc => {
                            state.search_query.clear();
                            state.end_search();
                        }
                        KeyCode::Enter => {
                            state.end_search();
                        }
                        KeyCode::Backspace => {
                            state.search_pop_char();
                        }
                        KeyCode::Char(c) => {
                            state.search_push_char(c);
                        }
                        _ => {}
                    },
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            state.on_tick();
            last_tick = Instant::now();
        }

        if state.should_quit {
            break;
        }
    }

    terminal::restore()?;
    Ok(())
}
