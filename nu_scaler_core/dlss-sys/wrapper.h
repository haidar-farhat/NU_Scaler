// This wrapper header includes the necessary NVIDIA Streamline SDK headers.
// Bindgen will process this file.

#pragma once

// Main Streamline header from the specified SDK path
#include "C:/nvideasdk/bckup/Streamline/include/sl.h"

// Conditionally include DLSS-G header if the main sl.h defines SL_DLSS_G
// and the header file actually exists at the specified path.
#if defined(SL_DLSS_G) && __has_include("C:/nvideasdk/bckup/Streamline/include/sl_dlss_g.h")
#include "C:/nvideasdk/bckup/Streamline/include/sl_dlss_g.h"
#endif

// If you need other specific Streamline headers (e.g., sl_reflex.h, sl_nis.h, etc.)
// that are not transitively included by sl.h, add them here using a similar
// absolute path and conditional inclusion pattern if appropriate.
// Example:
// #if defined(SL_REFLEX_ENABLED) && __has_include("C:/nvideasdk/bckup/Streamline/include/sl_reflex.h")
// #include "C:/nvideasdk/bckup/Streamline/include/sl_reflex.h"
// #endif

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