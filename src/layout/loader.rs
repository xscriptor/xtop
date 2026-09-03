use crate::layout::{Direction, LayoutArea, LayoutConstraint, LayoutDef, LayoutNode};
use std::fs;
use std::path::Path;

fn strip_jsonc_comments(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '/' && i + 1 < chars.len() {
            if chars[i + 1] == '/' {
                i += 2;
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
                continue;
            }
            if chars[i + 1] == '*' {
                i += 2;
                while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '/') {
                    i += 1;
                }
                i += 2;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

fn load_layout_from_file(path: &Path) -> Option<LayoutDef> {
    let data = fs::read_to_string(path).ok()?;
    let cleaned = strip_jsonc_comments(&data);
    serde_json::from_str::<LayoutDef>(&cleaned).ok()
}

pub fn layouts_dir() -> std::path::PathBuf {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        std::path::PathBuf::from(xdg).join("xtop").join("layouts")
    } else if let Ok(home) = std::env::var("HOME") {
        std::path::PathBuf::from(home)
            .join(".config")
            .join("xtop")
            .join("layouts")
    } else {
        std::path::PathBuf::from(".")
            .join(".config")
            .join("xtop")
            .join("layouts")
    }
}

pub fn load_custom_layouts() -> Vec<LayoutDef> {
    let dir = layouts_dir();
    if !dir.exists() {
        return vec![];
    }
    let mut layouts = vec![];
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str());
            if ext == Some("json") || ext == Some("jsonc") {
                if let Some(layout) = load_layout_from_file(&path) {
                    layouts.push(layout);
                }
            }
        }
    }
    layouts
}

pub fn builtin_layouts() -> Vec<LayoutDef> {
    vec![
        dashboard_layout(),
        vertical_layout(),
        horizontal_layout(),
        cpu_focus_layout(),
        memory_focus_layout(),
        network_focus_layout(),
        process_focus_layout(),
    ]
}

fn area(size: u16, node: LayoutNode) -> LayoutArea {
    LayoutArea {
        constraint: LayoutConstraint::Length(size),
        node,
    }
}

fn pct(pct: u16, node: LayoutNode) -> LayoutArea {
    LayoutArea {
        constraint: LayoutConstraint::Percentage(pct),
        node,
    }
}

fn fill(node: LayoutNode) -> LayoutArea {
    LayoutArea {
        constraint: LayoutConstraint::Fill,
        node,
    }
}

fn widget(name: &str) -> LayoutNode {
    LayoutNode::Widget {
        name: name.to_string(),
    }
}

fn split_h(areas: Vec<LayoutArea>) -> LayoutNode {
    LayoutNode::Split {
        direction: Direction::Horizontal,
        areas,
    }
}

fn split_v(areas: Vec<LayoutArea>) -> LayoutNode {
    LayoutNode::Split {
        direction: Direction::Vertical,
        areas,
    }
}

fn dashboard_layout() -> LayoutDef {
    LayoutDef {
        name: "Dashboard".into(),
        root: split_v(vec![
            area(3, widget("header")),
            pct(
                45,
                split_h(vec![
                    pct(50, widget("cpu")),
                    pct(
                        50,
                        split_v(vec![
                            pct(33, widget("memory")),
                            pct(33, widget("storage")),
                            pct(34, widget("network")),
                        ]),
                    ),
                ]),
            ),
            pct(52, widget("processes")),
        ]),
    }
}

fn vertical_layout() -> LayoutDef {
    LayoutDef {
        name: "Vertical".into(),
        root: split_v(vec![
            area(3, widget("header")),
            area(8, widget("cpu")),
            area(8, widget("memory")),
            area(6, widget("storage")),
            area(5, widget("network")),
            fill(widget("processes")),
        ]),
    }
}

fn horizontal_layout() -> LayoutDef {
    LayoutDef {
        name: "Horizontal".into(),
        root: split_v(vec![
            area(3, widget("header")),
            fill(split_h(vec![
                pct(25, widget("cpu")),
                pct(25, widget("memory")),
                pct(25, widget("storage")),
                pct(25, widget("network")),
            ])),
        ]),
    }
}

fn cpu_focus_layout() -> LayoutDef {
    LayoutDef {
        name: "CPU Focus".into(),
        root: split_v(vec![
            area(3, widget("header")),
            pct(60, widget("cpu")),
            fill(widget("processes")),
        ]),
    }
}

fn memory_focus_layout() -> LayoutDef {
    LayoutDef {
        name: "Memory Focus".into(),
        root: split_v(vec![
            area(3, widget("header")),
            pct(60, widget("memory")),
            fill(widget("processes")),
        ]),
    }
}

fn network_focus_layout() -> LayoutDef {
    LayoutDef {
        name: "Network Focus".into(),
        root: split_v(vec![
            area(3, widget("header")),
            pct(
                50,
                split_h(vec![pct(50, widget("network")), pct(50, widget("disk_io"))]),
            ),
            fill(widget("processes")),
        ]),
    }
}

fn process_focus_layout() -> LayoutDef {
    LayoutDef {
        name: "Process Focus".into(),
        root: split_v(vec![
            area(3, widget("header")),
            area(
                8,
                split_h(vec![
                    pct(25, widget("cpu")),
                    pct(25, widget("memory")),
                    pct(25, widget("storage")),
                    pct(25, widget("network")),
                ]),
            ),
            fill(widget("processes")),
        ]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_layouts_count() {
        let layouts = builtin_layouts();
        assert_eq!(layouts.len(), 7);
    }

    #[test]
    fn test_builtin_names() {
        let layouts = builtin_layouts();
        let names: Vec<&str> = layouts.iter().map(|l| l.name.as_str()).collect();
        assert!(names.contains(&"Dashboard"));
        assert!(names.contains(&"Vertical"));
        assert!(names.contains(&"CPU Focus"));
        assert!(names.contains(&"Process Focus"));
    }

    #[test]
    fn test_load_layout_from_jsonc() {
        let jsonc = r#"{
            // my custom layout
            "name": "test",
            "root": {
                "direction": "vertical",
                "areas": [
                    { "widget": "header", "size": 3 },
                    { "widget": "cpu", "size": "*" }
                ]
            }
        }"#;
        let cleaned = strip_jsonc_comments(jsonc);
        let layout: LayoutDef = serde_json::from_str(&cleaned).unwrap();
        assert_eq!(layout.name, "test");
    }
}
