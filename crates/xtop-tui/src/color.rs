use ratatui::prelude::Color;

pub fn to_color(c: &[u8; 3]) -> Color {
    Color::Rgb(c[0], c[1], c[2])
}

/// Returns a palette index for gauge color based on percentage:
/// - <50%  → green (2)
/// - 50–79% → yellow (3)
/// - ≥80%  → red (1)
pub fn gauge_gradient(pct: f64, alert_at: f64) -> usize {
    if pct >= alert_at {
        1
    } else if pct >= 50.0 {
        3
    } else {
        2
    }
}
