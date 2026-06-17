use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::fs;
use std::time::{Duration, Instant};
use xtop_core::application::state::{AppState, Config, InputMode, PalettePage};
use xtop_core::domain::keybinding::Action;
use xtop_core::infrastructure::config;
use xtop_core::infrastructure::layout_loader;
use xtop_core::infrastructure::sysinfo_provider::SysinfoProvider;
use xtop_core::infrastructure::theme_loader::load_all_themes;
use xtop_tui::render;
use xtop_tui::terminal;

fn key_event_to_str(key: &KeyEvent) -> String {
    let mut s = String::new();
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        s.push_str("ctrl+");
    }
    if key.modifiers.contains(KeyModifiers::ALT) {
        s.push_str("alt+");
    }
    match key.code {
        KeyCode::Char(c) => s.push(c),
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
const DEFAULT_THEMES: &[(&str, &str)] = &[
    ("x", include_str!("../../../assets/themes/x.jsonc")),
    ("madrid", include_str!("../../../assets/themes/madrid.jsonc")),
    ("lahabana", include_str!("../../../assets/themes/lahabana.jsonc")),
    ("paris", include_str!("../../../assets/themes/paris.jsonc")),
    ("tokio", include_str!("../../../assets/themes/tokio.jsonc")),
    ("oslo", include_str!("../../../assets/themes/oslo.jsonc")),
    ("helsinki", include_str!("../../../assets/themes/helsinki.jsonc")),
    ("berlin", include_str!("../../../assets/themes/berlin.jsonc")),
    ("london", include_str!("../../../assets/themes/london.jsonc")),
    ("praha", include_str!("../../../assets/themes/praha.jsonc")),
    ("bogota", include_str!("../../../assets/themes/bogota.jsonc")),
];

const DEFAULT_LAYOUTS: &[(&str, &str)] = &[
    ("dashboard", include_str!("../../../assets/layouts/dashboard.jsonc")),
    ("vertical", include_str!("../../../assets/layouts/vertical.jsonc")),
    ("horizontal", include_str!("../../../assets/layouts/horizontal.jsonc")),
    ("cpu_focus", include_str!("../../../assets/layouts/cpu_focus.jsonc")),
    ("memory_focus", include_str!("../../../assets/layouts/memory_focus.jsonc")),
    ("network_focus", include_str!("../../../assets/layouts/network_focus.jsonc")),
    ("process_focus", include_str!("../../../assets/layouts/process_focus.jsonc")),
];

fn ensure_default_assets() {
    let theme_assets: &[(&str, &str)] = DEFAULT_THEMES;
    let layout_assets: &[(&str, &str)] = DEFAULT_LAYOUTS;

    let dir = xtop_core::infrastructure::theme_loader::themes_dir();
    if !dir.join(".xtop_initialized").exists() {
        fs::create_dir_all(&dir).ok();
        for (name, content) in theme_assets {
            let path = dir.join(format!("{name}.jsonc"));
            if !path.exists() {
                fs::write(&path, content).ok();
            }
        }
        fs::write(dir.join(".xtop_initialized"), "").ok();
    }

    let dir = xtop_core::infrastructure::layout_loader::layouts_dir();
    if !dir.join(".xtop_initialized").exists() {
        fs::create_dir_all(&dir).ok();
        for (name, content) in layout_assets {
            let path = dir.join(format!("{name}.jsonc"));
            if !path.exists() {
                fs::write(&path, content).ok();
            }
        }
        fs::write(dir.join(".xtop_initialized"), "").ok();
    }
}

fn save_config(state: &AppState) {
    let cfg = Config {
        theme: state.current_theme.name.clone(),
        layout_mode: state.save_layout_mode(),
        update_interval_ms: state.update_interval_ms,
        history_points: 100,
        alerts: state.alerts,
        keybindings: state.keybindings.clone(),
    };
    let _ = config::save_config(&cfg);
}

fn main() -> anyhow::Result<()> {
    ensure_default_assets();

    terminal::install_panic_hook();
    let mut terminal = terminal::init()?;

    let provider = SysinfoProvider::new();
    let themes = load_all_themes();
    let cfg = config::load_config();
    let mut builtin_layouts = layout_loader::builtin_layouts();
    let custom_layouts = layout_loader::load_custom_layouts();
    builtin_layouts.extend(custom_layouts);
    let mut state = AppState::new(Box::new(provider), themes, cfg, builtin_layouts);

    let tick_rate = Duration::from_millis(state.update_interval_ms);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| render::render(f, &state))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_default();

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                let key_str = key_event_to_str(&key);
                match state.input_mode {
                    InputMode::Normal => {
                        if let Some(action) = state.keybindings.resolve(&key_str) {
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
                                    if action == Action::Quit {
                                        save_config(&state);
                                    }
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
