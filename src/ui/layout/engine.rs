//! Layout render engine: splits rects and dispatches widget renderers.
//!
//! Widgets live in packs (see the `widgets` repo); the kernel resolves
//! `(pack, name)` at render time. Plugin widgets ([`PluginWidget`]) keep
//! precedence over packs and can replace any name. Unknown names are
//! reported once per process (see [`warn_unknown_widget`]).

use crate::state::AppState;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::Frame;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};
use xtop_layout::{Direction, LayoutArea, LayoutConstraint, LayoutDef, LayoutNode};
use xtop_plugin_api::PluginWidget;
use xtop_widget_api::WidgetRenderer;

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
    plugin_widgets: &HashMap<String, PluginWidget>,
) {
    warn_unknown_widgets(state, def, plugin_widgets);
    render_node(f, state, area, &def.root, plugin_widgets);
}

/// Render a single named widget (used by fullscreen and minimal views).
/// Returns false when no renderer is registered for the name.
pub fn render_named(
    f: &mut Frame,
    state: &AppState,
    name: &str,
    area: Rect,
    plugin_widgets: &HashMap<String, PluginWidget>,
) -> bool {
    if let Some(widget) = plugin_widgets.get(name) {
        (widget.render)(f, state, area);
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
    plugin_widgets: &HashMap<String, PluginWidget>,
) {
    match node {
        LayoutNode::Widget { name } => {
            render_named(f, state, name, area, plugin_widgets);
        }
        LayoutNode::Split { direction, areas } => {
            if areas.is_empty() {
                return;
            }
            let constraints: Vec<Constraint> = areas.iter().map(to_ratatui_constraint).collect();
            let chunks = match direction {
                Direction::Horizontal => Layout::horizontal(constraints).split(area),
                Direction::Vertical => Layout::vertical(constraints).split(area),
            };
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

// ---------------------------------------------------------------------------
// Unknown-widget reporting (DR-3): a layout referencing a name no pack or
// plugin provides renders an empty area; that is a user-facing mistake, so
// the kernel warns once per widget name per process.
// ---------------------------------------------------------------------------

/// Widget names already reported as unknown this process.
static REPORTED_UNKNOWN_WIDGETS: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

/// Emit at most one stderr warning per unknown widget name per process.
fn warn_unknown_widget(layout: &str, name: &str) {
    if report_unknown_widget(name) {
        eprintln!("xtop: layout '{layout}' references unknown widget '{name}'");
    }
}

/// Mark a widget name as reported. Returns `true` on the first call for a
/// given name, `false` for every later call (the one-warning-per-name
/// guarantee). The set never shrinks, so the render path only pays for a
/// name once per process.
fn report_unknown_widget(name: &str) -> bool {
    let reported = REPORTED_UNKNOWN_WIDGETS.get_or_init(|| Mutex::new(HashSet::new()));
    let Ok(mut seen) = reported.lock() else {
        return true;
    };
    seen.insert(name.to_string())
}

/// Walk the layout tree once per frame and warn about leaf names no pack or
/// plugin can render. Cheap (pure name collection over a small tree); the
/// per-name dedup keeps repeated frames silent.
fn warn_unknown_widgets(
    state: &AppState,
    def: &LayoutDef,
    plugin_widgets: &HashMap<String, PluginWidget>,
) {
    fn walk<'a>(node: &'a LayoutNode, names: &mut Vec<&'a str>) {
        match node {
            LayoutNode::Widget { name } => names.push(name),
            LayoutNode::Split { areas, .. } => {
                for area in areas {
                    walk(&area.node, names);
                }
            }
        }
    }

    let mut names = Vec::new();
    walk(&def.root, &mut names);
    for name in names {
        if !plugin_widgets.contains_key(name) && resolve(state, name).is_none() {
            warn_unknown_widget(&def.name, name);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{report_unknown_widget, REPORTED_UNKNOWN_WIDGETS};
    use std::collections::HashSet;
    use std::sync::Mutex;

    #[test]
    fn unknown_widget_warns_once_per_name() {
        // Reset the process-wide set so this test is order-independent.
        REPORTED_UNKNOWN_WIDGETS
            .set(Mutex::new(HashSet::new()))
            .ok();
        // First report of a name returns true (a warning is emitted)...
        assert!(report_unknown_widget("ghost"));
        // ...every later report of the same name is silent.
        assert!(!report_unknown_widget("ghost"));
        // A different name still warns.
        assert!(report_unknown_widget("phantom"));
    }
}
