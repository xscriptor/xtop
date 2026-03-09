# Xtop Roadmap

This document outlines the development roadmap for **xtop**, a modern, cross-platform TUI system monitor.
It is synced automatically with GitHub Issues.

## Phase 1: Core Features <!-- phase:core -->

- [x] Basic TUI structure using `ratatui` and `crossterm`
- [x] System information collection using `sysinfo`
- [x] CPU usage per core and maximum temperature sensing
- [x] Memory and Swap monitoring with historical graphing
- [x] Network upload and download tracking
- [x] Storage usage visualization
- [x] Top 50 process list sorted by CPU usage
- [x] Cross-platform compatibility (macOS, Linux, Windows)

## Phase 2: Interface & Theming <!-- phase:ui-themes -->

- [x] Dynamic layout manager supporting multiple modes (Dashboard, Vertical, Process Focus)
- [x] Implement 13 built-in color schemes (`x`, `madrid`, `tokio`, etc.)
- [x] Instant theme and layout cycling at runtime
- [x] Responsive design for narrow terminals

## Phase 3: Deployment & Distribution <!-- phase:deployment -->

- [x] Automated install/uninstall scripts for Linux and macOS (`install.sh`, `uninstall.sh`)
- [x] Automated install/uninstall scripts for Windows (`install.ps1`, `uninstall.ps1`)
- [x] Quick curl/wget installation commands
- [ ] CI/CD pipeline for automated multi-platform binary releases
- [ ] Distribution packages (AUR, Homebrew, Winget, APT)

## Phase 4: Configuration & Customization <!-- phase:config -->

- [ ] Persistent configuration file support (save theme and layout preferences)
- [ ] Custom user theme creation via configuration
- [ ] Configurable update intervals for system metrics
- [ ] Customizable keybindings

## Phase 5: Advanced Monitoring Features <!-- phase:advanced-monitoring -->

- [ ] Disk I/O read/write speed tracking
- [ ] Granular network interface selection
- [ ] GPU usage, temperature, and VRAM monitoring (NVIDIA/AMD)
- [ ] Battery status monitoring
- [ ] Docker container resource usage integration

## Phase 6: Interactive Process Management <!-- phase:process-management -->

- [ ] Interactive process termination (send kill signals)
- [ ] Search, filter, and highlight processes by name
- [ ] Tree view for process hierarchy
- [ ] Sorting processes by Memory, PID, or User

## Phase 7: X Integration <!-- phase:x-integration -->

- [ ] X Integration
- [ ] Create xp package
- [ ] Add to X Repositories


