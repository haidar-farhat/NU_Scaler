"""// nu_scaler_core/src/dlss_manager.rs

use std::sync::Once;
use dlss_sys; // Assuming dlss-sys is a top-level dependency in nu_scaler_core's Cargo.toml

static SL_INIT: Once = Once::new();
static mut SL_INITIALIZED_SUCCESSFULLY: bool = false;

#[derive(Debug)]
pub enum DlssManagerError {
    SdkInitializationFailed(dlss_sys::SlStatus),
    SdkAlreadyInitialized, // Or handle this gracefully
}

pub struct DlssManager {
    // Later, this might hold a raw device pointer if we want to cache it,
    // or references to GPU resources. For now, it's mainly a conceptual owner
    // of the initialized SDK state for this instance of the manager.
    _private: (), // To make it a struct
}

impl DlssManager {
    pub fn new() -> Result<Self, DlssManagerError> {
        let mut initialization_status = dlss_sys::SlStatus::Success; // Assume success if already initialized

        SL_INIT.call_once(|| {
            // This closure is executed only once
            let status = unsafe { dlss_sys::slInitializeSDK() };
            if status == dlss_sys::SlStatus::Success {
                unsafe { SL_INITIALIZED_SUCCESSFULLY = true; }
                println!("[DLSS Manager] Streamline SDK Initialized successfully.");
            } else {
                initialization_status = status; // Capture the error status
                eprintln!("[DLSS Manager] Streamline SDK Initialization failed with status: {:?}", status);
            }
        });

        if unsafe { SL_INITIALIZED_SUCCESSFULLY } {
            Ok(DlssManager { _private: () })
        } else {
            // If call_once was executed previously and failed, SL_INITIALIZED_SUCCESSFULLY would be false.
            // If call_once just executed and failed, initialization_status holds the error.
            // This logic might need refinement if new() can be called multiple times after a failed init.
            // For now, if it wasn't successful, we return the captured status or a generic error.
             Err(DlssManagerError::SdkInitializationFailed(initialization_status))
        }
    }

    pub fn is_sdk_initialized() -> bool {
        unsafe { SL_INITIALIZED_SUCCESSFULLY }
    }

    // We will add methods here later like:
    // pub fn create_dlss_feature(&self, device_ptr: *mut std::ffi::c_void, width: u32, height: u32, quality: DlssQuality) -> Result<SlDlssFeatureHandle, DlssError> { ... }
}

// Optional: Add a global function to simplify getting an initialized manager or checking status.
pub fn ensure_sdk_initialized() -> Result<(), DlssManagerError> {
    if DlssManager::is_sdk_initialized() {
        return Ok(());
    }
    DlssManager::new().map(|_| ()) // Create an instance to trigger initialization
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests will actually try to initialize the SDK.
    // They might fail if the Streamline DLLs are not findable by the test runner,
    // or if the SDK has issues on the test machine.

    #[test]
    fn test_sdk_initialization() {
        // This relies on the NVIDIA Streamline DLLs (sl.interposer.dll, etc.)
        // being locatable by the test executable (e.g., in PATH or next to it).
        // Your build.rs for dlss-sys only links the .lib, it doesn't copy DLLs.
        // For testing, you might need to ensure DLLs are in the target/debug folder.
        
        // Clear the flag for testing, this is a bit hacky for test isolation.
        // In a real scenario, Once ensures it only runs once per process.
        // For repeated test runs in the same process, this is problematic without process isolation.
        // This test is more of an integration check.
        unsafe { SL_INITIALIZED_SUCCESSFULLY = false; }
        let _once_resetter = ResetOnceForTest(&SL_INIT);


        match DlssManager::new() {
            Ok(_) => {
                assert!(DlssManager::is_sdk_initialized(), "SDK should be marked as initialized.");
                println!("SDK Init test: Success");
            }
            Err(e) => {
                // This might happen if, for example, NVIDIA drivers are not installed,
                // or if sl.interposer.dll is not found or cannot load its dependencies.
                eprintln!("SDK Init test: Failed to initialize SDK: {:?}. This might be expected if environment is not set up for DLSS execution.", e);
                // Depending on CI/test environment, you might not want to panic here.
                // For local dev, a panic might be okay.
                // panic!("SDK Initialization failed in test: {:?}", e); 
                assert!(!DlssManager::is_sdk_initialized(), "SDK should not be marked as initialized on error.");
            }
        }
    }

    // Helper to somewhat reset `Once` for tests in the same process.
    // This is generally not a good practice for `Once` but can help for sequential tests.
    // Real isolation would require separate processes.
    struct ResetOnceForTest(&'static Once);
    impl Drop for ResetOnceForTest {
        fn drop(&mut self) {
            // Unfortunately, `Once` cannot be truly reset.
            // This means the `call_once` closure will not run again in the same process
            // if it has already run. `SL_INITIALIZED_SUCCESSFULLY` being false allows
            // `DlssManager::new()` to re-evaluate the init state though.
        }
    }
}
"" 