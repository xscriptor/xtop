pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;
    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    format!("{:.2} {}", size, UNITS[unit_idx])
}

pub fn format_uptime(secs: u64) -> String {
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;
    format!("{}d {}h {}m {}s", days, hours, minutes, seconds)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes_bytes() {
        assert_eq!(format_bytes(0), "0.00 B");
        assert_eq!(format_bytes(500), "500.00 B");
    }

    #[test]
    fn test_format_bytes_kb() {
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(2048), "2.00 KB");
        assert_eq!(format_bytes(1536), "1.50 KB");
    }

    #[test]
    fn test_format_bytes_mb() {
        assert_eq!(format_bytes(1048576), "1.00 MB");
        assert_eq!(format_bytes(3145728), "3.00 MB");
    }

    #[test]
    fn test_format_bytes_gb() {
        assert_eq!(format_bytes(1073741824), "1.00 GB");
        let two_gb = 2u64 * 1024 * 1024 * 1024;
        assert_eq!(format_bytes(two_gb), "2.00 GB");
    }

    #[test]
    fn test_format_bytes_tb() {
        let one_tb = 1024u64 * 1024 * 1024 * 1024;
        assert_eq!(format_bytes(one_tb), "1.00 TB");
    }

    #[test]
    fn test_format_uptime_zero() {
        assert_eq!(format_uptime(0), "0d 0h 0m 0s");
    }

    #[test]
    fn test_format_uptime_full() {
        let secs = 1 + 60 * 2 + 3600 * 3 + 86400 * 4; // 4d 3h 2m 1s
        assert_eq!(format_uptime(secs), "4d 3h 2m 1s");
    }

    #[test]
    fn test_format_uptime_seconds_only() {
        assert_eq!(format_uptime(59), "0d 0h 0m 59s");
    }

    #[test]
    fn test_format_uptime_exact_hour() {
        assert_eq!(format_uptime(3600), "0d 1h 0m 0s");
    }

    #[test]
    fn test_format_uptime_exact_day() {
        assert_eq!(format_uptime(86400), "1d 0h 0m 0s");
    }
}
