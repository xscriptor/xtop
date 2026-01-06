# xtop

**xtop** is a modern, cross-platform terminal-based system monitor written in Rust. It is heavily inspired by `btop` but built with the efficiency and safety of Rust, utilizing `ratatui` for the interface and `sysinfo` for system data.

## Features

- **Cross-Platform:** Runs on macOS, Linux, and Windows.
- **System Monitoring:**
  - **CPU:** Usage per core/thread, maximum temperature sensing.
  - **Memory:** RAM and Swap usage with historical graphing.
  - **Network:** Real-time upload and download tracking.
  - **Disks:** Storage usage visualization.
  - **Processes:** List of running processes sorted by CPU usage.
- **Theming:**
  - Includes 13 built-in color schemes (e.g., Dracula-like 'x', Madrid, Tokio, etc.).
  - Cycle through themes instantly without configuration files.
- **Layouts:**
  - **Dashboard:** Balanced view of all components (Default).
  - **Vertical:** Stacked view, good for narrow terminals.
  - **Process Focus:** Maximizes space for the process list while keeping essential stats visible.

## Installation

### Quick Install (macOS/Linux)

You can install `xtop` directly using our installer script. This requires `rust` (cargo) to be installed on your system.

**Install:**
```bash
curl -fsSL https://raw.githubusercontent.com/yourusername/xtop/main/install.sh | bash
```

**Uninstall:**
```bash
curl -fsSL https://raw.githubusercontent.com/yourusername/xtop/main/uninstall.sh | bash
```

### Quick Install (Windows PowerShell)

Requires [Rust (Cargo)](https://rustup.rs/) installed. Run in PowerShell:

**Install:**
```powershell
irm https://raw.githubusercontent.com/yourusername/xtop/main/install.ps1 | iex
```

**Uninstall:**
```powershell
irm https://raw.githubusercontent.com/yourusername/xtop/main/uninstall.ps1 | iex
```

### Prerequisites
- [Rust & Cargo](https://www.rust-lang.org/tools/install) installed.

### Build from Source

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/xtop.git
   cd xtop
   ```

2. Build and run:
   ```bash
   cargo run --release
   ```

## Usage

### Keybindings

| Key | Action |
| --- | --- |
| `q` | Quit application |
| `t` | Next Color Theme |
| `T` | Previous Color Theme |
| `l` | Toggle Layout Mode (Dashboard -> Vertical -> Process Focus) |

### Modules

1. **Header**: Shows system uptime, load average, current theme, and layout mode.
2. **CPU**: Shows usage bars for each CPU core. If sensors are available, shows the maximum CPU temperature.
3. **Memory**: Gauges for RAM and Swap usage, plus a line chart for RAM history.
4. **Network**: Total downloaded (RX) and uploaded (TX) data.
5. **Processes**: A scrolling list of the top 50 processes sorted by CPU usage.

## Configuration

Currently, `xtop` is zero-config. All preferences (theme, layout) can be toggled at runtime but are reset on restart. Future versions may include a config file.

## License

MIT
