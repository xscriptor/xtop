# Xtop Roadmap

This document outlines the development roadmap for **xtop**, a modern, cross-platform TUI system monitor.
It is synced automatically with GitHub Issues.

## Phase 1: Core Features <!-- phase:core -->

- [x] Basic TUI structure using `ratatui` and `crossterm` (#1)
- [x] System information collection using `sysinfo` (#2)
- [x] CPU usage per core and maximum temperature sensing (#3)
- [x] Memory and Swap monitoring with historical graphing (#4)
- [x] Network upload and download tracking (#5)
- [x] Storage usage visualization (#6)
- [x] Top 50 process list sorted by CPU usage (#7)
- [x] Cross-platform compatibility (macOS, Linux, Windows) (#8)

## Phase 2: Interface & Theming <!-- phase:ui-themes -->

- [x] Dynamic layout manager supporting multiple modes (Dashboard, Vertical, Process Focus) (#9)
- [x] Implement 13 built-in color schemes (`x`, `madrid`, `tokio`, etc.) (#10)
- [x] Instant theme and layout cycling at runtime (#11)
- [x] Responsive design for narrow terminals (#12)

## Phase 3: Deployment & Distribution <!-- phase:deployment -->

- [x] Automated install/uninstall scripts for Linux and macOS (`install.sh`, `uninstall.sh`) (#13)
- [x] Automated install/uninstall scripts for Windows (`install.ps1`, `uninstall.ps1`) (#14)
- [x] Quick curl/wget installation commands (#15)
- [ ] CI/CD pipeline for automated multi-platform binary releases (#16)
- [ ] Distribution packages (AUR, Homebrew, Winget, APT) (#17)

## Phase 4: Configuration & Customization <!-- phase:config -->

- [ ] Persistent configuration file support (save theme and layout preferences) (#18)
- [ ] Custom user theme creation via configuration (#19)
- [ ] Configurable update intervals for system metrics (#20)
- [ ] Customizable keybindings (#21)

## Phase 5: Advanced Monitoring Features <!-- phase:advanced-monitoring -->

- [ ] Disk I/O read/write speed tracking (#22)
- [ ] Granular network interface selection (#23)
- [ ] GPU usage, temperature, and VRAM monitoring (NVIDIA/AMD) (#24)
- [ ] Battery status monitoring (#25)
- [ ] Docker container resource usage integration (#26)

## Phase 6: Interactive Process Management <!-- phase:process-management -->

- [ ] Interactive process termination (send kill signals) (#27)
- [ ] Search, filter, and highlight processes by name (#28)
- [ ] Tree view for process hierarchy (#29)
- [ ] Sorting processes by Memory, PID, or User (#30)

## Phase 7: X Integration <!-- phase:x-integration -->

- [ ] X Integration (#31)
- [ ] Create xp package (#32)
- [ ] Add to X Repositories (#33)


