// nu_scaler_core/src/dlss_manager.rs

use dlss_sys;
use std::sync::Once; // Assuming dlss-sys is a top-level dependency in nu_scaler_core's Cargo.toml

static SL_INIT: Once = Once::new();
static mut SL_INITIALIZED_SUCCESSFULLY: bool = false;

#[derive(Debug)]
pub enum DlssManagerError {
    SdkInitializationFailed(dlss_sys::SlStatus),
    // SdkAlreadyInitialized, // Decided to let Once handle this; new() will just return Ok if already init.
}

pub struct DlssManager {
    // This struct primarily serves to gate access to DLSS features
    // ensuring the SDK is initialized.
    _private: (),
}

impl DlssManager {
    pub fn new() -> Result<Self, DlssManagerError> {
        let mut captured_status_on_fail = dlss_sys::SlStatus::Success; // Placeholder for error from closure

        SL_INIT.call_once(|| {
            // This closure is executed only once per process.
            let status = unsafe { dlss_sys::slInitializeSDK() };
            if status == dlss_sys::SlStatus::Success {
                unsafe {
                    SL_INITIALIZED_SUCCESSFULLY = true;
                }
                println!(
                    "[DLSS Manager] Streamline SDK Initialized successfully via slInitializeSDK()."
                );
            } else {
                captured_status_on_fail = status; // Capture the error status
                eprintln!(
                    "[DLSS Manager] Streamline SDK Initialization failed with status: {:?}",
                    status
                );
            }
        });

        if unsafe { SL_INITIALIZED_SUCCESSFULLY } {
            Ok(DlssManager { _private: () })
        } else {
            // This means the call_once executed (or had executed previously) and failed.
            Err(DlssManagerError::SdkInitializationFailed(
                captured_status_on_fail,
            ))
        }
    }

    /// Checks if the global SDK initialization has been successfully performed.
    pub fn is_sdk_initialized() -> bool {
        // Check if Once has run and if it was successful.
        SL_INIT.is_completed() && unsafe { SL_INITIALIZED_SUCCESSFULLY }
    }

    // TODO: Add methods like:
    // pub fn create_dlss_feature(&self, device_ptr: *mut std::ffi::c_void, width: u32, height: u32 /*, other_options */)
    //   -> Result<SlDlssFeatureHandle, DlssFeatureError>
    // {
    //   if !Self::is_sdk_initialized() { return Err(DlssFeatureError::SdkNotInitialized); }
    //   // ... actual call to dlss_sys::slCreateDlssFeature ...
    // }
}

/// Ensures the Streamline SDK is initialized.
/// Call this once before any DLSS operations if you're not using a DlssManager instance directly.
pub fn ensure_sdk_initialized() -> Result<(), DlssManagerError> {
    if DlssManager::is_sdk_initialized() {
        return Ok(());
    }
    // Attempt to initialize by creating a manager instance.
    DlssManager::new().map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    // IMPORTANT: These tests interact with global state (SL_INIT, SL_INITIALIZED_SUCCESSFULLY)
    // and perform actual FFI calls. They are more like integration tests.
    // Running `cargo test` will execute these.
    //
    // To make these tests work:
    // 1. Ensure NVIDIA drivers supporting DLSS/Streamline are installed.
    // 2. Ensure the Streamline SDK DLLs (e.g., sl.interposer.dll and its dependencies)
    //    are accessible to the test executable. This usually means they need to be:
    //    a) In a directory listed in the system's PATH environment variable.
    //    b) In the same directory as the test executable (e.g., C:\Nu_Scaler\NU_Scaler\target\debug\).
    //       Your `dlss-sys/build.rs` links the .lib but doesn't copy DLLs. You might need a
    //       post-build step for your main application, and for tests, you might need to
    //       manually copy them or adjust PATH for the test environment.

    #[test]
    fn test_initial_sdk_initialization_attempt() {
        // This test assumes it might be the first one to run and trigger initialization.
        // Or, if another test ran `ensure_sdk_initialized` or `DlssManager::new()`,
        // it will check the outcome of that single global initialization.

        println!("Attempting to ensure SDK is initialized via ensure_sdk_initialized()...");
        match ensure_sdk_initialized() {
            Ok(()) => {
                println!("test_initial_sdk_initialization_attempt: SDK initialized or was already initialized successfully.");
                assert!(
                    DlssManager::is_sdk_initialized(),
                    "SDK should be marked as initialized."
                );
            }
            Err(e) => {
                eprintln!("test_initial_sdk_initialization_attempt: SDK initialization failed: {:?}. This can be normal if the environment (drivers, DLLs) isn't set up for DLSS execution.", e);
                assert!(
                    !DlssManager::is_sdk_initialized(),
                    "SDK should NOT be marked as initialized on error."
                );
                // In a CI environment or for specific test setups, you might choose to panic or not.
                // For now, we'll just assert the state.
                // panic!("SDK Initialization failed, which might be an issue depending on the test environment: {:?}", e);
            }
        }
    }

    #[test]
    fn test_subsequent_initialization_calls_do_not_reinitialize() {
        // Ensures that after the first attempt (successful or not by SL_INIT),
        // further calls to DlssManager::new() or ensure_sdk_initialized()
        // reflect the outcome of that first attempt without re-running the slInitializeSDK FFI.

        println!("First ensure_sdk_initialized call (may trigger actual init if not done yet)...");
        let _ = ensure_sdk_initialized(); // Outcome handled in the test above or by SL_INIT state

        let first_call_state = DlssManager::is_sdk_initialized();
        println!("SDK state after first ensure: {}", first_call_state);

        println!("Calling ensure_sdk_initialized() again...");
        let result_second_call = ensure_sdk_initialized();
        let second_call_state = DlssManager::is_sdk_initialized();
        println!("SDK state after second ensure: {}", second_call_state);

        assert_eq!(
            first_call_state, second_call_state,
            "SDK initialization state should remain consistent."
        );

        if first_call_state {
            assert!(
                result_second_call.is_ok(),
                "If SDK was init, subsequent calls should also be Ok."
            );
        } else {
            // If the first init failed, subsequent attempts to get a manager will also fail,
            // reflecting that initial permanent failure for the process.
            assert!(
                result_second_call.is_err(),
                "If SDK init failed, subsequent calls should also reflect error."
            );
        }

        // Try creating a DlssManager instance directly
        match DlssManager::new() {
            Ok(_) => {
                assert!(first_call_state, "If DlssManager::new() succeeded, SDK should have been marked initialized from the first attempt.");
            }
            Err(_) => {
                assert!(!first_call_state, "If DlssManager::new() failed, SDK should have been marked as not initialized from the first attempt.");
            }
        }
    }
}
