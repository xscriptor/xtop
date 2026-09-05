//! Linux probes for the sysinfo area.
//!
//! Each subsystem lives in its own file and is re-exported here, so the
//! platform dispatcher only talks to this module.

mod battery;
mod governor;
mod gpu;
mod interfaces;
mod mounts;
mod power;
mod temps;
mod threads;

pub use battery::*;
pub use governor::*;
pub use gpu::*;
pub use interfaces::*;
pub use mounts::*;
pub use power::*;
pub use temps::*;
pub use threads::*;

/// Directory Services users: not applicable on Linux; the kernel reads the
/// full user table from `/etc/passwd` itself.
pub fn read_directory_users() -> Vec<(u32, String)> {
    Vec::new()
}
