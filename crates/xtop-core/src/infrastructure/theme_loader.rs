use crate::domain::theme::Theme;
use std::fs;
use std::path::Path;

fn make_theme(name: &str, colors: [&str; 16]) -> Theme {
    let mut palette = [[0u8; 3]; 16];
    for (i, h) in colors.iter().enumerate() {
        palette[i] = crate::domain::theme::hex_to_rgb_pub(h);
    }
    Theme {
        name: name.to_string(),
        palette,
    }
}

fn default_theme() -> Theme {
    make_theme(
        "x",
        [
            "#050505", "#fc618d", "#7bd88f", "#fce566", "#fd9353", "#948ae3", "#5ad4e6",
            "#f7f1ff", "#0f0f0f", "#fc618d", "#7bd88f", "#fce566", "#fd9353", "#948ae3",
            "#5ad4e6", "#f7f1ff",
        ],
    )
}

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

fn load_theme_from_file(path: &Path) -> Option<Theme> {
    let data = fs::read_to_string(path).ok()?;
    let cleaned = strip_jsonc_comments(&data);
    serde_json::from_str::<Theme>(&cleaned).ok()
}

fn load_themes_from_dir(dir: &Path) -> Vec<Theme> {
    if !dir.exists() {
        return vec![];
    }
    let mut themes = vec![];
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str());
            if ext == Some("json") || ext == Some("jsonc") {
                if let Some(theme) = load_theme_from_file(&path) {
                    themes.push(theme);
                }
            }
        }
    }
    themes
}

pub fn themes_dir() -> std::path::PathBuf {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        std::path::PathBuf::from(xdg).join("xtop").join("themes")
    } else if let Ok(home) = std::env::var("HOME") {
        std::path::PathBuf::from(home)
            .join(".config")
            .join("xtop")
            .join("themes")
    } else {
        std::path::PathBuf::from(".")
            .join(".config")
            .join("xtop")
            .join("themes")
    }
}

pub fn load_all_themes() -> Vec<Theme> {
    let mut themes = vec![default_theme()];

    let user_dir = themes_dir();
    let custom = load_themes_from_dir(&user_dir);
    for t in custom {
        if !themes.iter().any(|existing| existing.name == t.name) {
            themes.push(t);
        }
    }

    themes
}

pub fn builtin_themes() -> Vec<Theme> {
    vec![default_theme()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_theme() {
        let t = default_theme();
        assert_eq!(t.name, "x");
        assert_eq!(t.bg(), &[5, 5, 5]);
    }

    #[test]
    fn test_builtin_themes_count() {
        let themes = builtin_themes();
        assert_eq!(themes.len(), 1);
    }

    #[test]
    fn test_strip_line_comment() {
        let input = "{\"name\": \"test\" // comment\n}";
        let result = strip_jsonc_comments(input);
        assert_eq!(result, "{\"name\": \"test\" \n}");
    }

    #[test]
    fn test_strip_block_comment() {
        let input = "{\"name\": /* comment */ \"test\"}";
        let result = strip_jsonc_comments(input);
        assert_eq!(result, "{\"name\":  \"test\"}");
    }

    #[test]
    fn test_strip_no_comments() {
        let input = "{\"name\": \"test\"}";
        let result = strip_jsonc_comments(input);
        assert_eq!(result, "{\"name\": \"test\"}");
    }

    #[test]
    fn test_hex_to_rgb() {
        let result = crate::domain::theme::hex_to_rgb_pub("#ff0000");
        assert_eq!(result, [255, 0, 0]);
    }
}
