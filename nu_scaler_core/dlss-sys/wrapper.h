// This wrapper header includes the necessary NVIDIA Streamline SDK headers.
// Bindgen will process this file.

#pragma once

// Use an absolute path to the new SDK 2.7.30 include directory
#include "C:/nvideasdk/bckup/Streamline/include/sl.h"

// If these were previously included from the old SDK path, update them too.
// Assuming they are also in C:/nvideasdk/bckup/Streamline/include/
// If not, adjust or remove as necessary.
#if SL_DLSS_G
#include "C:/nvideasdk/bckup/Streamline/include/sl_dlss_g.h" 
#endif

// Example for other features if needed, ensure these files exist at the new path
// #include "C:/nvideasdk/bckup/Streamline/include/sl_reflex.h"

// Ensure that the build script (build.rs) can find these headers.
// The current build.rs doesn't explicitly add include paths to bindgen,
// so absolute paths here are the most reliable for now.

// Main Streamline header
// It's common for SDK headers to be in a subdirectory of the main include path.
// e.g. C:/nvideasdk/bckup/Streamline/include/Streamline/sl.h
// If sl.h is directly in C:/nvideasdk/bckup/Streamline/include, then "sl.h" is correct.
// Otherwise, adjust the path like "Streamline/sl.h"
#include "sl.h" 

// Streamline DLSS specific header (if needed and not pulled in by sl.h)
// Check if this is also in a subdirectory like "Streamline/sl_dlss.h"
#include "sl_dlss.h" 

// Other common Streamline headers you might need, depending on features used:
// #include "sl_reflex.h" // Or "Streamline/sl_reflex.h"
// #include "sl_common.h" // Or "Streamline/sl_common.h"

#endif // DLSS_STREAMLINE_WRAPPER_H 