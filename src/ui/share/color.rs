//! Color conversion helpers used by kernel UI chrome (overlays, minimal view).

use ratatui::prelude::Color;

pub fn to_color(c: &[u8; 3]) -> Color {
    Color::Rgb(c[0], c[1], c[2])
}
