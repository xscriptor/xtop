use crate::domain::theme::Theme;

fn hex_to_rgb(hex: &str) -> [u8; 3] {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(hex.get(0..2).unwrap_or("00"), 16).unwrap_or(0);
    let g = u8::from_str_radix(hex.get(2..4).unwrap_or("00"), 16).unwrap_or(0);
    let b = u8::from_str_radix(hex.get(4..6).unwrap_or("00"), 16).unwrap_or(0);
    [r, g, b]
}

fn make_theme(name: &str, colors: [&str; 16]) -> Theme {
    let mut palette = [[0u8; 3]; 16];
    for (i, h) in colors.iter().enumerate() {
        palette[i] = hex_to_rgb(h);
    }
    Theme {
        name: name.to_string(),
        palette,
    }
}

pub fn builtin_themes() -> Vec<Theme> {
    vec![
        make_theme(
            "x",
            [
                "#363537", "#fc618d", "#7bd88f", "#fce566", "#fd9353", "#948ae3", "#5ad4e6",
                "#f7f1ff", "#69676c", "#fc618d", "#7bd88f", "#fce566", "#fd9353", "#948ae3",
                "#5ad4e6", "#f7f1ff",
            ],
        ),
        make_theme(
            "madrid",
            [
                "#333333", "#cc0033", "#009933", "#b8860b", "#0099cc", "#6633cc", "#0099cc",
                "#1a1a1a", "#666666", "#cc0033", "#009933", "#b8860b", "#0099cc", "#6633cc",
                "#0099cc", "#1a1a1a",
            ],
        ),
        make_theme(
            "lahabana",
            [
                "#363537", "#fc618d", "#7bd88f", "#e5ff9d", "#fd9353", "#948ae3", "#5ad4e6",
                "#f7f1ff", "#69676c", "#fc618d", "#7bd88f", "#e5ff9d", "#fd9353", "#948ae3",
                "#5ad4e6", "#f7f1ff",
            ],
        ),
        make_theme(
            "seul",
            [
                "#1b1b1b", "#FF4C8B", "#7FFFD4", "#FFD84C", "#00FFA8", "#D36CFF", "#47CFFF",
                "#f7f1ff", "#69676c", "#FF4C8B", "#7FFFD4", "#FFD84C", "#00FFA8", "#D36CFF",
                "#47CFFF", "#f7f1ff",
            ],
        ),
        make_theme(
            "miami",
            [
                "#000000", "#FF4C8B", "#7FFFD4", "#FFD84C", "#00FFA8", "#D36CFF", "#47CFFF",
                "#f7f1ff", "#69676c", "#FF4C8B", "#7FFFD4", "#FFD84C", "#00FFA8", "#D36CFF",
                "#47CFFF", "#f7f1ff",
            ],
        ),
        make_theme(
            "paris",
            [
                "#222222", "#fc618d", "#7bd88f", "#fce566", "#a3f3ff", "#c4bdff", "#a3f3ff",
                "#f7f1ff", "#525053", "#fc618d", "#7bd88f", "#fce566", "#a3f3ff", "#c4bdff",
                "#a3f3ff", "#f7f1ff",
            ],
        ),
        make_theme(
            "tokio",
            [
                "#363537", "#fc618d", "#7bd88f", "#fce566", "#fd9353", "#948ae3", "#5ad4e6",
                "#f7f1ff", "#69676c", "#fc618d", "#7bd88f", "#fce566", "#fd9353", "#948ae3",
                "#5ad4e6", "#f7f1ff",
            ],
        ),
        make_theme(
            "oslo",
            [
                "#3f4451", "#e05561", "#8cc265", "#d18f52", "#4aa5f0", "#c162de", "#42b3c2",
                "#e6e6e6", "#4f5666", "#ff616e", "#a5e075", "#f0a45d", "#4dc4ff", "#de73ff",
                "#4cd1e0", "#ffffff",
            ],
        ),
        make_theme(
            "helsinki",
            [
                "#c0bbae", "#1faa9e", "#733d9a", "#2e70ad", "#b55a0f", "#3e9d21", "#bd4c3d",
                "#191919", "#b0a999", "#009e91", "#5a1f8a", "#0f5ba2", "#b23b00", "#218c00",
                "#b32e1f", "#000000",
            ],
        ),
        make_theme(
            "berlin",
            [
                "#000000", "#999999", "#bbbbbb", "#dddddd", "#888888", "#aaaaaa", "#cccccc",
                "#ffffff", "#333333", "#bbbbbb", "#dddddd", "#ffffff", "#aaaaaa", "#cccccc",
                "#eeeeee", "#ffffff",
            ],
        ),
        make_theme(
            "london",
            [
                "#000000", "#333333", "#444444", "#555555", "#666666", "#777777", "#888888",
                "#999999", "#333333", "#444444", "#555555", "#666666", "#777777", "#888888",
                "#999999", "#aaaaaa",
            ],
        ),
        make_theme(
            "praha",
            [
                "#1A1A1A", "#FF5555", "#B8E6A0", "#FFE4A3", "#BD93F9", "#FF9AA2", "#8BE9FD",
                "#FFFFFF", "#6272A4", "#FF6E6E", "#B8E6A0", "#FFE4A3", "#D6ACFF", "#FF9AA2",
                "#A4FFFF", "#FFFFFF",
            ],
        ),
        make_theme(
            "bogota",
            [
                "#222222", "#fc618d", "#7bd88f", "#ffed89", "#47e6ff", "#ff9999", "#47e6ff",
                "#f7f1ff", "#525053", "#fc618d", "#7bd88f", "#ffed89", "#47e6ff", "#ff9999",
                "#47e6ff", "#f7f1ff",
            ],
        ),
    ]
}

pub fn load_all_themes() -> Vec<Theme> {
    let mut themes = builtin_themes();
    let custom = crate::infrastructure::config::load_custom_themes();
    for t in custom {
        if !themes.iter().any(|existing| existing.name == t.name) {
            themes.push(t);
        }
    }
    themes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_to_rgb_basic() {
        assert_eq!(hex_to_rgb("#ff0000"), [255, 0, 0]);
        assert_eq!(hex_to_rgb("#00ff00"), [0, 255, 0]);
        assert_eq!(hex_to_rgb("#0000ff"), [0, 0, 255]);
    }

    #[test]
    fn test_hex_to_rgb_black_white() {
        assert_eq!(hex_to_rgb("#000000"), [0, 0, 0]);
        assert_eq!(hex_to_rgb("#ffffff"), [255, 255, 255]);
    }

    #[test]
    fn test_hex_to_rgb_without_hash() {
        assert_eq!(hex_to_rgb("ff0000"), [255, 0, 0]);
    }

    #[test]
    fn test_hex_to_rgb_invalid() {
        assert_eq!(hex_to_rgb("#xyz"), [0, 0, 0]); // unwrap_or(0)
    }

    #[test]
    fn test_hex_to_rgb_short() {
        assert_eq!(hex_to_rgb("#ff"), [255, 0, 0]); // only 2 chars
    }

    #[test]
    fn test_builtin_themes_count() {
        let themes = builtin_themes();
        assert_eq!(themes.len(), 13);
    }

    #[test]
    fn test_builtin_themes_have_names() {
        let themes = builtin_themes();
        let names: Vec<&str> = themes.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"x"));
        assert!(names.contains(&"madrid"));
        assert!(names.contains(&"tokio"));
        assert!(names.contains(&"praha"));
    }

    #[test]
    fn test_builtin_themes_palette_size() {
        let themes = builtin_themes();
        for theme in &themes {
            assert_eq!(
                theme.palette.len(),
                16,
                "Theme '{}' should have 16 palette entries",
                theme.name
            );
        }
    }

    #[test]
    fn test_builtin_theme_bg_fg() {
        let themes = builtin_themes();
        let x = themes.iter().find(|t| t.name == "x").unwrap();
        assert_eq!(x.bg(), &[0x36, 0x35, 0x37]);
        assert_eq!(x.fg(), &[0xf7, 0xf1, 0xff]);
    }

    #[test]
    fn test_hex_to_rgb_uppercase() {
        assert_eq!(hex_to_rgb("#FF0000"), [255, 0, 0]);
        assert_eq!(hex_to_rgb("#AABBCC"), [170, 187, 204]);
    }
}
