//! Layout render engine: splits rects and dispatches widget renderers.
//!
//! Widgets live in packs (see the `widgets` repo); the kernel resolves
//! `(pack, name)` at render time. Plugin widgets keep precedence over packs
//! and can replace any name.

use crate::state::AppState;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::Frame;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use xtop_layout::{Direction, LayoutArea, LayoutConstraint, LayoutDef, LayoutNode};
use xtop_plugin_api::HostState;
use xtop_widget_api::WidgetRenderer;

/// A plugin widget renderer (plugins see only the API contract).
pub type PluginWidgetFn = Arc<dyn Fn(&mut Frame, &dyn HostState, Rect) + Send + Sync>;

/// One compiled-in widget pack.
struct Pack {
    name: &'static str,
    renderers: &'static HashMap<&'static str, WidgetRenderer>,
}

static BASE_PACK: OnceLock<HashMap<&'static str, WidgetRenderer>> = OnceLock::new();
#[cfg(feature = "widget-blocks")]
static BLOCKS_PACK: OnceLock<HashMap<&'static str, WidgetRenderer>> = OnceLock::new();

/// The packs compiled into this binary, in precedence order.
fn packs() -> &'static [Pack] {
    static PACKS: OnceLock<Vec<Pack>> = OnceLock::new();
    PACKS.get_or_init(|| {
        // `mut` is only used when the blocks pack is compiled in.
        #[cfg_attr(not(feature = "widget-blocks"), allow(unused_mut))]
        let mut v = vec![Pack {
            name: "default",
            renderers: BASE_PACK.get_or_init(xtop_widgets::registry),
        }];
        #[cfg(feature = "widget-blocks")]
        v.push(Pack {
            name: "blocks",
            renderers: BLOCKS_PACK.get_or_init(xtop_widget_blocks::registry),
        });
        v
    })
}

/// Resolve the renderer for a widget name following the user's pack choice
/// (`style.pack` global or per-widget `style.widgets.<name>.pack`). Unknown
/// packs and names gracefully fall back to the base pack.
fn resolve(state: &AppState, name: &str) -> Option<&'static WidgetRenderer> {
    let packs = packs();
    let chosen = state.style.pack_for(name);
    if let Some(pack_name) = chosen {
        if let Some(pack) = packs.iter().find(|p| p.name == pack_name) {
            if let Some(r) = pack.renderers.get(name) {
                return Some(r);
            }
        }
    }
    packs
        .iter()
        .find(|p| p.name == "default")
        .and_then(|p| p.renderers.get(name))
}

/// Render a layout definition within a given area.
///
/// `plugin_widgets` is an optional extension from plugins; plugin widgets
/// take precedence over every pack.
pub fn render_layout(
    f: &mut Frame,
    state: &AppState,
    area: Rect,
    def: &LayoutDef,
    plugin_widgets: &HashMap<String, PluginWidgetFn>,
) {
    render_node(f, state, area, &def.root, plugin_widgets);
}

/// Render a single named widget (used by fullscreen and minimal views).
/// Returns false when no renderer is registered for the name.
pub fn render_named(
    f: &mut Frame,
    state: &AppState,
    name: &str,
    area: Rect,
    plugin_widgets: &HashMap<String, PluginWidgetFn>,
) -> bool {
    if let Some(render_fn) = plugin_widgets.get(name) {
        render_fn(f, state, area);
        return true;
    }
    if let Some(render_fn) = resolve(state, name) {
        render_fn(f, state, area);
        return true;
    }
    false
}

fn render_node(
    f: &mut Frame,
    state: &AppState,
    area: Rect,
    node: &LayoutNode,
    plugin_widgets: &HashMap<String, PluginWidgetFn>,
) {
    match node {
        LayoutNode::Widget { name } => {
            render_named(f, state, name, area, plugin_widgets);
        }
        LayoutNode::Split { direction, areas } => {
            if areas.is_empty() {
                return;
            }
            let dir = match direction {
                Direction::Horizontal => ratatui::prelude::Direction::Horizontal,
                Direction::Vertical => ratatui::prelude::Direction::Vertical,
            };
            let constraints: Vec<Constraint> = areas.iter().map(to_ratatui_constraint).collect();
            let chunks = Layout::default()
                .direction(dir)
                .constraints(constraints)
                .split(area);
            for (i, chunk) in chunks.iter().enumerate() {
                if i < areas.len() {
                    render_node(f, state, *chunk, &areas[i].node, plugin_widgets);
                }
            }
        }
    }
}

fn to_ratatui_constraint(area: &LayoutArea) -> Constraint {
    match area.constraint {
        LayoutConstraint::Length(n) => Constraint::Length(n),
        LayoutConstraint::Percentage(p) => Constraint::Percentage(p),
        LayoutConstraint::Fill => Constraint::Fill(1),
    }
}
