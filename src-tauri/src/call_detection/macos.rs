use std::ffi::c_void;
use std::mem;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use super::CallDetector;

/// Polling is metadata-only (a device running-state flag), not audio
/// capture, but it has not been empirically verified whether reading it
/// triggers a mic-permission (TCC) prompt on newer macOS versions — worth
/// checking on the target OS before shipping.
const POLL_INTERVAL: Duration = Duration::from_secs(5);

// Hand-rolled CoreAudio FFI: only a handful of symbols are needed here, so
// this avoids pulling in a bindgen-based sys crate (which requires a
// libclang matching the host toolchain's architecture — a real source of
// build friction that isn't worth it for four constants and one function).
type AudioObjectId = u32;

#[repr(C)]
struct AudioObjectPropertyAddress {
    selector: u32,
    scope: u32,
    element: u32,
}

const K_AUDIO_OBJECT_SYSTEM_OBJECT: AudioObjectId = 1;
const K_AUDIO_OBJECT_PROPERTY_ELEMENT_MAIN: u32 = 0;
const K_AUDIO_OBJECT_PROPERTY_SCOPE_GLOBAL: u32 = u32::from_be_bytes(*b"glob");
const K_AUDIO_HARDWARE_PROPERTY_DEFAULT_INPUT_DEVICE: u32 = u32::from_be_bytes(*b"dIn ");
const K_AUDIO_DEVICE_PROPERTY_DEVICE_IS_RUNNING_SOMEWHERE: u32 = u32::from_be_bytes(*b"goin");

#[link(name = "CoreAudio", kind = "framework")]
extern "C" {
    fn AudioObjectGetPropertyData(
        in_object_id: AudioObjectId,
        in_address: *const AudioObjectPropertyAddress,
        in_qualifier_data_size: u32,
        in_qualifier_data: *const c_void,
        io_data_size: *mut u32,
        out_data: *mut c_void,
    ) -> i32;
}

pub struct MacCallDetector {
    cached: AtomicBool,
    last_poll: Mutex<Instant>,
}

impl MacCallDetector {
    pub fn new() -> Self {
        Self {
            cached: AtomicBool::new(false),
            last_poll: Mutex::new(Instant::now() - POLL_INTERVAL),
        }
    }

    fn poll(&self) -> bool {
        unsafe {
            let mut device_id: AudioObjectId = 0;
            let mut size = mem::size_of::<AudioObjectId>() as u32;
            let device_addr = AudioObjectPropertyAddress {
                selector: K_AUDIO_HARDWARE_PROPERTY_DEFAULT_INPUT_DEVICE,
                scope: K_AUDIO_OBJECT_PROPERTY_SCOPE_GLOBAL,
                element: K_AUDIO_OBJECT_PROPERTY_ELEMENT_MAIN,
            };
            let status = AudioObjectGetPropertyData(
                K_AUDIO_OBJECT_SYSTEM_OBJECT,
                &device_addr,
                0,
                std::ptr::null(),
                &mut size,
                &mut device_id as *mut AudioObjectId as *mut c_void,
            );
            if status != 0 || device_id == 0 {
                return false;
            }

            let mut running: u32 = 0;
            let mut running_size = mem::size_of::<u32>() as u32;
            let running_addr = AudioObjectPropertyAddress {
                selector: K_AUDIO_DEVICE_PROPERTY_DEVICE_IS_RUNNING_SOMEWHERE,
                scope: K_AUDIO_OBJECT_PROPERTY_SCOPE_GLOBAL,
                element: K_AUDIO_OBJECT_PROPERTY_ELEMENT_MAIN,
            };
            let status = AudioObjectGetPropertyData(
                device_id,
                &running_addr,
                0,
                std::ptr::null(),
                &mut running_size,
                &mut running as *mut u32 as *mut c_void,
            );
            status == 0 && running != 0
        }
    }
}

impl Default for MacCallDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl CallDetector for MacCallDetector {
    fn is_active(&self) -> bool {
        let mut last = self.last_poll.lock().unwrap();
        if last.elapsed() >= POLL_INTERVAL {
            let result = self.poll();
            self.cached.store(result, Ordering::Relaxed);
            *last = Instant::now();
        }
        self.cached.load(Ordering::Relaxed)
    }
}
