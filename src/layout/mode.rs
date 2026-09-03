//! Layout modes and effective-layout detection.
//!
//! How the requested layout mode degrades depending on terminal size.

use crate::layout::LayoutDef;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum LayoutMode {
    Dashboard,
    Vertical,
    Horizontal,
    CpuFocus,
    MemoryFocus,
    NetworkFocus,
    ProcessFocus,
}

impl LayoutMode {
    #[cfg(test)]
    pub fn next(self) -> Self {
        match self {
            Self::Dashboard => Self::Vertical,
            Self::Vertical => Self::Horizontal,
            Self::Horizontal => Self::CpuFocus,
            Self::CpuFocus => Self::MemoryFocus,
            Self::MemoryFocus => Self::NetworkFocus,
            Self::NetworkFocus => Self::ProcessFocus,
            Self::ProcessFocus => Self::Dashboard,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Dashboard => "Dashboard",
            Self::Vertical => "Vertical",
            Self::Horizontal => "Horizontal",
            Self::CpuFocus => "CPU Focus",
            Self::MemoryFocus => "Memory Focus",
            Self::NetworkFocus => "Network Focus",
            Self::ProcessFocus => "Process Focus",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EffectiveLayout {
    Dashboard,
    Compact,
    Vertical,
    Horizontal,
    CpuFocus,
    MemoryFocus,
    NetworkFocus,
    ProcessFocus,
    Minimal,
}

pub(crate) fn layout_index_from_mode(mode: LayoutMode, defs: &[LayoutDef]) -> usize {
    let label = mode.label();
    defs.iter().position(|d| d.name == label).unwrap_or(0)
}

pub(crate) fn mode_from_layout_index(index: usize) -> LayoutMode {
    match index {
        0 => LayoutMode::Dashboard,
        1 => LayoutMode::Vertical,
        2 => LayoutMode::Horizontal,
        3 => LayoutMode::CpuFocus,
        4 => LayoutMode::MemoryFocus,
        5 => LayoutMode::NetworkFocus,
        6 => LayoutMode::ProcessFocus,
        _ => LayoutMode::Dashboard,
    }
}

pub fn detect_effective_layout(width: u16, height: u16, user_mode: LayoutMode) -> EffectiveLayout {
    if width < 60 || height < 14 {
        return EffectiveLayout::Minimal;
    }
    match user_mode {
        LayoutMode::Dashboard => {
            if width < 80 {
                EffectiveLayout::Vertical
            } else if width < 100 || height < 28 {
                EffectiveLayout::Compact
            } else {
                EffectiveLayout::Dashboard
            }
        }
        LayoutMode::Vertical => EffectiveLayout::Vertical,
        LayoutMode::Horizontal => EffectiveLayout::Horizontal,
        LayoutMode::CpuFocus => EffectiveLayout::CpuFocus,
        LayoutMode::MemoryFocus => EffectiveLayout::MemoryFocus,
        LayoutMode::NetworkFocus => EffectiveLayout::NetworkFocus,
        LayoutMode::ProcessFocus => EffectiveLayout::ProcessFocus,
    }
}
