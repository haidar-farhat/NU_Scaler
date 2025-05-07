// This wrapper header includes the necessary NVIDIA Streamline SDK headers.
// Bindgen will process this file.

#ifndef DLSS_STREAMLINE_WRAPPER_H
#define DLSS_STREAMLINE_WRAPPER_H

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