// nu_scaler_core/src/dlss_manager.rs

use dlss_sys::{self, SlStatus}; // Assuming dlss-sys is a top-level dependency
use std::sync::OnceLock;
use log::{debug, error, info}; // Added log import

// Static OnceLock to store the result of the first initialization attempt.
static SL_INIT_RESULT: OnceLock<Result<(), DlssManagerError>> = OnceLock::new();

#[derive(Debug, Clone)] // Added Clone
pub enum DlssManagerError {
    SdkInitializationFailed(SlStatus),
    SymbolLoadingFailed(String), // New variant for symbol loading issues
}

// This function performs the actual SDK initialization.
// It's called by OnceLock::get_or_init.
fn perform_initialization() -> Result<(), DlssManagerError> {
    info!("[DLSS Manager] Attempting Streamline SDK initialization...");
    match dlss_sys::slInitializeSDK() { // This is the wrapper from dlss-sys crate
        std::result::Result::Ok(status) => { // Fully qualified Ok
            if status == dlss_sys::SlStatus::Success { // Fully qualified SlStatus for comparison
                info!("[DLSS Manager] Streamline SDK initialized successfully via slInitializeSDK().");
                Ok(())
            } else {
                error!("[DLSS Manager] slInitializeSDK() reported failure status: {:?}", status);
                Err(DlssManagerError::SdkInitializationFailed(status))
            }
        }
        std::result::Result::Err(load_error) => { // Fully qualified Err
            // load_error is &'static dlss_sys::LoadError
            let error_message = load_error.0.clone(); // Assuming LoadError(String)
            error!("[DLSS Manager] Failed to load slInitializeSDK symbol: {}", error_message);
            Err(DlssManagerError::SymbolLoadingFailed(error_message))
        }
    }
}

/// Ensures the Streamline SDK is initialized, performing initialization if it hasn't occurred yet.
/// Returns the result of the initialization attempt.
pub fn ensure_sdk_initialized() -> Result<(), DlssManagerError> {
    // .clone() is used because get_or_init returns a &Result<...>, and we need to propagate
    // an owned Result (specifically the Err variant if it occurs).
    SL_INIT_RESULT.get_or_init(perform_initialization).clone()
}

pub struct DlssManager {
    _private: (), // To ensure it's only constructed via new()
}

impl DlssManager {
    pub fn new() -> Result<Self, DlssManagerError> {
        ensure_sdk_initialized().map(|_| DlssManager { _private: () })
    }

    pub fn is_sdk_initialized() -> bool {
        SL_INIT_RESULT.get().map_or(false, Result::is_ok)
    }

    // TODO: Add methods like:
    // pub fn create_dlss_feature(&self, device_ptr: *mut std::ffi::c_void, width: u32, height: u32 /*, other_options */)
    //   -> Result<SlDlssFeatureHandle, DlssFeatureError>
    // {
    //   if !Self::is_sdk_initialized() { return Err(DlssFeatureError::SdkNotInitialized); }
    //   // ... actual call to dlss_sys::slCreateDlssFeature ...
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to reset OnceLock for isolated tests. This is generally not good practice
    // for production code but can be useful for making tests independent.
    // However, std::sync::OnceLock does not have a public reset mechanism.
    // Tests will rely on the fact that `cargo test` runs tests in separate threads or processes,
    // or that the OnceLock state persists correctly across test functions within the same process.
    // For robust testing of OnceLock, especially failure paths, specific test binaries
    // might be needed if tests interfere with each other's global state.

    #[test]
    fn test_initial_sdk_initialization_attempt() {
        info!("[Test] Running test_initial_sdk_initialization_attempt...");
        match ensure_sdk_initialized() {
            Ok(()) => {
                info!("[Test] SDK initialized or was already initialized successfully.");
                assert!(DlssManager::is_sdk_initialized(), "SDK should be marked as initialized.");
            }
            Err(e) => {
                error!("[Test] SDK initialization failed: {:?}. This can be normal if the environment (drivers, DLLs) isn't set up for DLSS execution.", e);
                assert!(!DlssManager::is_sdk_initialized(), "SDK should NOT be marked as initialized on error.");
            }
        }
    }

    #[test]
    fn test_subsequent_initialization_calls_reflect_first_attempt() {
        info!("[Test] Running test_subsequent_initialization_calls_reflect_first_attempt...");
        // Call once to establish the initial state (either by this test or a previous one)
        let first_result = ensure_sdk_initialized();
        let first_call_state_is_ok = first_result.is_ok();
        info!("[Test] SDK state after first ensure: {}", if first_call_state_is_ok { "Ok" } else { "Err" });

        info!("[Test] Calling ensure_sdk_initialized() again...");
        let second_result = ensure_sdk_initialized();
        let second_call_state_is_ok = second_result.is_ok();
        info!("[Test] SDK state after second ensure: {}", if second_call_state_is_ok { "Ok" } else { "Err" });

        assert_eq!(
            first_call_state_is_ok, second_call_state_is_ok,
            "SDK initialization state (Ok/Err) should remain consistent."
        );

        // Further checks based on the actual result type
        match first_result {
            Ok(_) => assert!(second_result.is_ok(), "If SDK was init Ok, subsequent calls should also be Ok."),
            Err(ref first_err) => {
                match second_result {
                    Ok(_) => panic!("If first init was Err, second should not be Ok."),
                    Err(ref second_err) => {
                        // Check if the errors are of the same type/content (requires PartialEq for DlssManagerError)
                         assert_eq!(format!("{:?}", first_err), format!("{:?}", second_err), "Errors should be consistent");
                    }
                }
            }
        }
        
        // Test DlssManager::new() behavior
        let manager_result = DlssManager::new();
        assert_eq!(manager_result.is_ok(), first_call_state_is_ok, "DlssManager::new() success should match initial ensure_sdk_initialized() success.");
    }
}
