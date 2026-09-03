//! The interactive TUI run loop.

use std::time::{Duration, Instant};

use crate::config::keybinding::Action;
use crate::state::{InputMode, PalettePage};
use crate::ui;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};

use super::share::{config_dir, initialize_state, save_config};

fn key_event_to_str(key: &KeyEvent) -> String {
    let mut s = String::new();
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    if ctrl {
        s.push_str("ctrl+");
    }
    if key.modifiers.contains(KeyModifiers::ALT) {
        s.push_str("alt+");
    }
    match key.code {
        KeyCode::Char(c) => {
            if ctrl {
                s.push(c.to_ascii_lowercase());
            } else {
                s.push(c);
            }
        }
        KeyCode::Esc => s.push_str("escape"),
        KeyCode::Enter => s.push_str("enter"),
        KeyCode::Backspace => s.push_str("backspace"),
        KeyCode::Tab => s.push_str("tab"),
        KeyCode::Up => s.push_str("up"),
        KeyCode::Down => s.push_str("down"),
        KeyCode::Left => s.push_str("left"),
        KeyCode::Right => s.push_str("right"),
        KeyCode::Delete => s.push_str("delete"),
        KeyCode::Home => s.push_str("home"),
        KeyCode::End => s.push_str("end"),
        KeyCode::PageUp => s.push_str("pageup"),
        KeyCode::PageDown => s.push_str("pagedown"),
        _ => return String::new(),
    }
    s
}

// Embedded default asset files (shipped with the binary)
/// Run the interactive TUI loop.
pub fn run() -> anyhow::Result<()> {
    ui::install_panic_hook();
    let mut terminal = ui::init()?;

    let cfg_dir = config_dir();
    let mut state = initialize_state(&cfg_dir)?;

    let tick_rate = Duration::from_millis(state.update_interval_ms);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui::render(f, &state))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_default();

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                let key_str = key_event_to_str(&key);

                // Give plugins first chance to consume the key
                let key_str_clone = key_str.clone();
                let key_consumed =
                    state.with_plugin_manager_mut(|mgr, this| mgr.handle_key(this, &key_str_clone));
                if key_consumed {
                    continue;
                }

                // DEBUG: print key for diagnostics
                if cfg!(debug_assertions) && !key_str.is_empty() {
                    eprintln!("[key] '{key_str}'");
                }

                match state.input_mode {
                    InputMode::Normal => {
                        // Direct Ctrl+P check (works regardless of keybinding config, important on macOS)
                        if key_str == "ctrl+p" {
                            state.open_palette();
                            state.input_mode = InputMode::CommandPalette;
                        } else if let Some(action) = state.keybindings.resolve(&key_str) {
                            match action {
                                Action::Quit => {
                                    save_config(&state);
                                    state.quit();
                                }
                                Action::Cancel if state.show_help => {
                                    state.toggle_help();
                                }
                                Action::OpenCommandPalette => {
                                    state.open_palette();
                                    state.input_mode = InputMode::CommandPalette;
                                }
                                Action::KillProcess | Action::ProcessUp | Action::ProcessDown => {
                                    state.execute_action(&action);
                                }
                                _ => {
                                    state.execute_action(&action);
                                }
                            }
                        }
                    }
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
                    InputMode::CommandPalette => {
                        let is_main = state.palette.page == PalettePage::Main;
                        match key.code {
                            KeyCode::Esc => {
                                state.close_palette();
                            }
                            KeyCode::Enter => {
                                if let Some(action) = state.palette_selected_action() {
                                    state.execute_action(&action);
                                    save_config(&state);
                                }
                            }
                            KeyCode::Down => {
                                state.palette_select_next();
                            }
                            KeyCode::Up => {
                                state.palette_select_prev();
                            }
                            KeyCode::Char(c) => {
                                state.palette.query.push(c);
                                state.palette_filter();
                            }
                            KeyCode::Backspace => {
                                if state.palette.query.is_empty() && !is_main {
                                    state.palette_navigate_to(PalettePage::Main);
                                } else {
                                    state.palette.query.pop();
                                    state.palette_filter();
                                }
                            }
                            _ => {}
                        }
                    }
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

    // Disable plugins on shutdown
    state.with_plugin_manager_mut(|mgr, this| {
        mgr.disable_all(this);
    });

    ui::restore()?;
    Ok(())
}
