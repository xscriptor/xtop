<h1 align="center"> Xtop </h1>

<div align="center">

![rust](https://xscriptor.github.io/badges/languages/rust.svg) ![mit](https://xscriptor.github.io/badges/licenses/mit.svg) ![shell](https://xscriptor.github.io/badges/languages/shell.svg) ![powershell](https://xscriptor.github.io/badges/languages/powershell.svg) ![xtop](https://xscriptor.github.io/badges/software/xtop.svg)

xtop is a modern, cross-platform TUI system monitor crafted in Rust. Heavily inspired by btop, it leverages Rust's safety and performance, powered by ratatui for the interface and sysinfo for real-time metrics.

</div>

<p align="center"><img src="./assets/icon.png" width="100" alt="Xscriptor logo" /></p>

---

# Previews

<p align="center">
  <a href="./assets/previews/preview1.png">
    <img src="./assets/previews/preview1.png" alt="Main preview" width="850"/>
  </a>
</p>

<details>
  <summary>More previews</summary>

  <table>
    <tr>
      <td align="center">
        <a href="./assets/previews/preview2.png">
          <img src="./assets/previews/preview2.png" alt="Preview 2" width="380"/>
        </a>
      </td>
      <td align="center">
        <a href="./assets/previews/preview3.png">
          <img src="./assets/previews/preview3.png" alt="Preview 3" width="380"/>
        </a>
      </td>
    </tr>
    <tr>
      <td align="center">
        <a href="./assets/previews/preview4.png">
          <img src="./assets/previews/preview4.png" alt="Preview 4" width="380"/>
        </a>
      </td>
      <td align="center">
      </td>
    </tr>
  </table>
</details>

## Features

- **Cross-Platform:** Runs on macOS, Linux, and Windows.
- **System Monitoring:**
  - **CPU:** Usage per core/thread, maximum temperature sensing.
  - **Memory:** RAM and Swap usage with historical graphing.
  - **Network:** Real-time upload and download tracking.
  - **Disks:** Storage usage visualization.
  - **Processes:** List of running processes sorted by CPU usage.
- **Theming:**
  - 13 ready-to-use color schemes + custom themes via JSONC files.
  - Cycle through themes instantly with `t` / `T`.
- **Layouts:**
  - 7 built-in layouts (Dashboard, Vertical, Horizontal, CPU/Memory/Network/Process Focus).
  - Custom layouts via JSONC files — define your own widget tree.
  - Full-screen mode for any widget.

## Installation

### Quick Install (macOS/Linux)

The installer script automatically detects your distribution and installs all required dependencies (including Rust if needed).

**Install with curl:**
```bash
curl -fsSL https://raw.githubusercontent.com/xscriptor/xtop/main/install.sh | bash
```

**Or with wget:**
```bash
wget -qO- https://raw.githubusercontent.com/xscriptor/xtop/main/install.sh | bash
```

**Uninstall:**
```bash
curl -fsSL https://raw.githubusercontent.com/xscriptor/xtop/main/install.sh | bash -s -- --uninstall
```

<details>
<summary>Installer Options</summary>

You can also run the installer with additional options:

```bash
# Check dependencies without installing
./install.sh --check-deps

# Install only dependencies (Rust, build tools)
./install.sh --install-deps

# Show help
./install.sh --help
```

**Supported distributions:** Arch, Debian/Ubuntu, Fedora/RHEL, openSUSE, Alpine, and derivatives.

</details>

### Quick Install (Windows PowerShell)

Requires [Rust (Cargo)](https://rustup.rs/) installed. Run in PowerShell:

**Install:**
```powershell
irm https://raw.githubusercontent.com/xscriptor/xtop/main/install.ps1 | iex
```

**Uninstall:**
```powershell
irm https://raw.githubusercontent.com/xscriptor/xtop/main/uninstall.ps1 | iex
```

### Build from Source

1. Clone the repository:
   ```bash
   git clone https://github.com/xscriptor/xtop.git
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
| `l` | Next Layout Mode (built-in + custom) |
| `f` | Toggle fullscreen widget |
| `F` | Cycle fullscreen widget |
| `/` | Search / filter processes |

### Modules

1. **Header**: Shows system uptime, load average, current theme, and layout mode.
2. **CPU**: Shows usage bars for each CPU core. If sensors are available, shows the maximum CPU temperature.
3. **Memory**: Gauges for RAM and Swap usage, plus a line chart for RAM history.
4. **Network**: Total downloaded (RX) and uploaded (TX) data.
5. **Processes**: A scrolling list of the top 50 processes sorted by CPU usage.

## Customization

xtop supports custom color themes and layout modes defined as JSONC files.

**[→ Full customization guide](docs/customization.md)**

| Feature | Location | Format |
|---------|----------|--------|
| Themes | `~/.config/xtop/themes/*.jsonc` | 16-entry hex color palette |
| Layouts | `~/.config/xtop/layouts/*.jsonc` | Recursive split/widget tree |

The built-in `miami` theme (black background, neon accents) is always available. Starter theme and layout files ship in the `assets/` directory.

## Contributing

Contributions are always welcome! Please read the [contribution guidelines](CONTRIBUTING.md) first.

## License
[MIT](LICENSE)

<div align="center">
<a href="https://github.com/xscriptor">---X---</a>
</div>