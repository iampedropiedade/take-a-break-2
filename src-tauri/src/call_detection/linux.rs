use super::CallDetector;

/// Stub for v1: always reports no call in progress. A real implementation
/// would shell out to `pactl list source-outputs` (PulseAudio) or
/// `wpctl status` / `pw-cli ls Node` (PipeWire) and parse for an active
/// recording stream — genuinely best-effort and fragile across distros, so
/// left off by default on Linux until revisited.
#[derive(Default)]
pub struct LinuxCallDetector;

impl CallDetector for LinuxCallDetector {
    fn is_active(&self) -> bool {
        false
    }
}
