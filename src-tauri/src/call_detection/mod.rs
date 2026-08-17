#[cfg(not(target_os = "macos"))]
pub mod linux;
#[cfg(target_os = "macos")]
pub mod macos;

use std::sync::Arc;

/// Best-effort "is the user currently on a call" check, used to gate
/// break-triggering when `Settings::cancel_on_call` is enabled. Implemented
/// as a mic-in-use heuristic — not a perfect signal (any mic use counts,
/// e.g. a voice memo), but requires no per-app integration.
pub trait CallDetector: Send + Sync {
    fn is_active(&self) -> bool;
}

pub fn platform_detector() -> Arc<dyn CallDetector> {
    #[cfg(target_os = "macos")]
    {
        Arc::new(macos::MacCallDetector::new())
    }
    #[cfg(not(target_os = "macos"))]
    {
        Arc::new(linux::LinuxCallDetector::default())
    }
}
