use ratatui::style::Color;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Theme {
    pub name: String,
    pub palette: [Color; 16],
}

impl Theme {
    pub fn bg(&self) -> Color {
        self.palette[0]
    }
    
    pub fn fg(&self) -> Color {
        self.palette[7]
    }
    
    #[allow(dead_code)]
    pub fn graph_colors(&self) -> Vec<Color> {
        vec![
            self.palette[1], // Red
            self.palette[2], // Green
            self.palette[3], // Yellow
            self.palette[4], // Blue
            self.palette[5], // Magenta
            self.palette[6], // Cyan
        ]
    }
}

fn hex_to_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    Color::Rgb(r, g, b)
}

pub fn get_themes() -> HashMap<String, Theme> {
    let mut themes = HashMap::new();

    let definitions = vec![
        ("x", vec![
            "#363537", "#fc618d", "#7bd88f", "#fce566", "#fd9353", "#948ae3", "#5ad4e6", "#f7f1ff",
            "#69676c", "#fc618d", "#7bd88f", "#fce566", "#fd9353", "#948ae3", "#5ad4e6", "#f7f1ff"
        ]),
        ("madrid", vec![
            "#333333", "#cc0033", "#009933", "#b8860b", "#0099cc", "#6633cc", "#0099cc", "#1a1a1a",
            "#666666", "#cc0033", "#009933", "#b8860b", "#0099cc", "#6633cc", "#0099cc", "#1a1a1a"
        ]),
        ("lahabana", vec![
            "#363537", "#fc618d", "#7bd88f", "#e5ff9d", "#fd9353", "#948ae3", "#5ad4e6", "#f7f1ff",
            "#69676c", "#fc618d", "#7bd88f", "#e5ff9d", "#fd9353", "#948ae3", "#5ad4e6", "#f7f1ff"
        ]),
        ("seul", vec![
            "#1b1b1b", "#FF4C8B", "#7FFFD4", "#FFD84C", "#00FFA8", "#D36CFF", "#47CFFF", "#f7f1ff",
            "#69676c", "#FF4C8B", "#7FFFD4", "#FFD84C", "#00FFA8", "#D36CFF", "#47CFFF", "#f7f1ff"
        ]),
        ("miami", vec![
            "#000000", "#FF4C8B", "#7FFFD4", "#FFD84C", "#00FFA8", "#D36CFF", "#47CFFF", "#f7f1ff",
            "#69676c", "#FF4C8B", "#7FFFD4", "#FFD84C", "#00FFA8", "#D36CFF", "#47CFFF", "#f7f1ff"
        ]),
        ("paris", vec![
            "#222222", "#fc618d", "#7bd88f", "#fce566", "#a3f3ff", "#c4bdff", "#a3f3ff", "#f7f1ff",
            "#525053", "#fc618d", "#7bd88f", "#fce566", "#a3f3ff", "#c4bdff", "#a3f3ff", "#f7f1ff"
        ]),
        ("tokio", vec![
            "#363537", "#fc618d", "#7bd88f", "#fce566", "#fd9353", "#948ae3", "#5ad4e6", "#f7f1ff",
            "#69676c", "#fc618d", "#7bd88f", "#fce566", "#fd9353", "#948ae3", "#5ad4e6", "#f7f1ff"
        ]),
        ("oslo", vec![
            "#3f4451", "#e05561", "#8cc265", "#d18f52", "#4aa5f0", "#c162de", "#42b3c2", "#e6e6e6",
            "#4f5666", "#ff616e", "#a5e075", "#f0a45d", "#4dc4ff", "#de73ff", "#4cd1e0", "#ffffff"
        ]),
        ("helsinki", vec![
            "#c0bbae", "#1faa9e", "#733d9a", "#2e70ad", "#b55a0f", "#3e9d21", "#bd4c3d", "#191919",
            "#b0a999", "#009e91", "#5a1f8a", "#0f5ba2", "#b23b00", "#218c00", "#b32e1f", "#000000"
        ]),
        ("berlin", vec![
            "#000000", "#999999", "#bbbbbb", "#dddddd", "#888888", "#aaaaaa", "#cccccc", "#ffffff",
            "#333333", "#bbbbbb", "#dddddd", "#ffffff", "#aaaaaa", "#cccccc", "#eeeeee", "#ffffff"
        ]),
        ("london", vec![
            "#000000", "#333333", "#444444", "#555555", "#666666", "#777777", "#888888", "#999999",
            "#333333", "#444444", "#555555", "#666666", "#777777", "#888888", "#999999", "#aaaaaa"
        ]),
        ("praha", vec![
            "#1A1A1A", "#FF5555", "#B8E6A0", "#FFE4A3", "#BD93F9", "#FF9AA2", "#8BE9FD", "#FFFFFF",
            "#6272A4", "#FF6E6E", "#B8E6A0", "#FFE4A3", "#D6ACFF", "#FF9AA2", "#A4FFFF", "#FFFFFF"
        ]),
        ("bogota", vec![
            "#222222", "#fc618d", "#7bd88f", "#ffed89", "#47e6ff", "#ff9999", "#47e6ff", "#f7f1ff",
            "#525053", "#fc618d", "#7bd88f", "#ffed89", "#47e6ff", "#ff9999", "#47e6ff", "#f7f1ff"
        ]),
    ];

    for (name, colors) in definitions {
        let mut palette = [Color::Reset; 16];
        for (i, hex) in colors.iter().enumerate() {
            if i < 16 {
                palette[i] = hex_to_color(hex);
            }
        }
        themes.insert(name.to_string(), Theme {
            name: name.to_string(),
            palette,
        });
    }

    themes
}
