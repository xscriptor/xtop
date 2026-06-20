use crate::render::{battery, cpu, disk_io, gpu, header, memory, network, processes, storage};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::Frame;
use std::collections::HashMap;
use std::sync::Arc;
use xtop_core::application::state::AppState;
use xtop_core::domain::layout::{Direction, LayoutArea, LayoutDef, LayoutNode};

/// A widget renderer: a callable that draws a widget onto the terminal.
pub type WidgetFn = Arc<dyn Fn(&mut Frame, &AppState, Rect) + Send + Sync>;

/// Create the default built-in widget map.
pub fn default_widgets() -> HashMap<&'static str, WidgetFn> {
    let mut m: HashMap<&'static str, WidgetFn> = HashMap::new();
    m.insert("header", Arc::new(header::render));
    m.insert("cpu", Arc::new(cpu::render));
    m.insert("memory", Arc::new(memory::render));
    m.insert("storage", Arc::new(storage::render));
    m.insert("network", Arc::new(network::render));
    m.insert("processes", Arc::new(processes::render));
    m.insert("disk_io", Arc::new(disk_io::render));
    m.insert("battery", Arc::new(battery::render));
    m.insert("gpu", Arc::new(gpu::render));
    m
}

/// Render a layout definition within a given area.
///
/// `widgets` is the built-in registry. `plugin_widgets` is an optional
/// extension from plugins. Plugin widgets take precedence over built-ins.
pub fn render_layout(
    f: &mut Frame,
    state: &AppState,
    area: Rect,
    def: &LayoutDef,
    widgets: &HashMap<&'static str, WidgetFn>,
    plugin_widgets: &HashMap<String, WidgetFn>,
) {
    render_node(f, state, area, &def.root, widgets, plugin_widgets);
}

fn render_node(
    f: &mut Frame,
    state: &AppState,
    area: Rect,
    node: &LayoutNode,
    widgets: &HashMap<&'static str, WidgetFn>,
    plugin_widgets: &HashMap<String, WidgetFn>,
) {
    match node {
        LayoutNode::Widget { name } => {
            // Plugin widgets take precedence
            if let Some(render_fn) = plugin_widgets.get(name) {
                render_fn(f, state, area);
            } else if let Some(render_fn) = widgets.get(name.as_str()) {
                render_fn(f, state, area);
            }
            // Unknown widgets are silently ignored (backward-compatible)
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
                    render_node(f, state, *chunk, &areas[i].node, widgets, plugin_widgets);
                }
            }
        }
    }
}

fn to_ratatui_constraint(area: &LayoutArea) -> Constraint {
    match area.constraint {
        xtop_core::domain::layout::LayoutConstraint::Length(n) => Constraint::Length(n),
        xtop_core::domain::layout::LayoutConstraint::Percentage(p) => Constraint::Percentage(p),
        xtop_core::domain::layout::LayoutConstraint::Fill => Constraint::Fill(1),
    }
}
