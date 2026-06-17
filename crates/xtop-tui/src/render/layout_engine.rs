use crate::render::{battery, cpu, disk_io, gpu, header, memory, network, processes, storage};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::Frame;
use std::collections::HashMap;
use xtop_core::domain::layout::{Direction, LayoutArea, LayoutDef, LayoutNode};
use xtop_core::application::state::AppState;

pub type WidgetRenderer = fn(&mut Frame, &AppState, Rect);

pub fn default_widgets() -> HashMap<&'static str, WidgetRenderer> {
    let mut m: HashMap<&'static str, WidgetRenderer> = HashMap::new();
    m.insert("header", header::render);
    m.insert("cpu", cpu::render);
    m.insert("memory", memory::render);
    m.insert("storage", storage::render);
    m.insert("network", network::render);
    m.insert("processes", processes::render);
    m.insert("disk_io", disk_io::render);
    m.insert("battery", battery::render);
    m.insert("gpu", gpu::render);
    m
}

pub fn render_layout(
    f: &mut Frame,
    state: &AppState,
    area: Rect,
    def: &LayoutDef,
    widgets: &HashMap<&'static str, WidgetRenderer>,
) {
    render_node(f, state, area, &def.root, widgets);
}

fn render_node(
    f: &mut Frame,
    state: &AppState,
    area: Rect,
    node: &LayoutNode,
    widgets: &HashMap<&'static str, WidgetRenderer>,
) {
    match node {
        LayoutNode::Widget { name } => {
            if let Some(render_fn) = widgets.get(name.as_str()) {
                render_fn(f, state, area);
            }
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
                    render_node(f, state, *chunk, &areas[i].node, widgets);
                }
            }
        }
    }
}

fn to_ratatui_constraint(area: &LayoutArea) -> Constraint {
    match area.constraint {
        xtop_core::domain::layout::LayoutConstraint::Length(n) => Constraint::Length(n),
        xtop_core::domain::layout::LayoutConstraint::Percentage(p) => Constraint::Percentage(p),
        xtop_core::domain::layout::LayoutConstraint::Fill => Constraint::Min(0),
    }
}
