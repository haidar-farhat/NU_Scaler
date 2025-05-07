// DLSS SDK wrapper header
#ifndef DLSS_WRAPPER_H
#define DLSS_WRAPPER_H

// Standard includes
#include <stdint.h>
#include <stdbool.h>

// Try to include DLSS SDK headers
#if __has_include(<nvsdk_ngx/nvsdk_ngx.h>)
    // Main NVIDIA NGX SDK header
    #include <nvsdk_ngx/nvsdk_ngx.h>
    
    // DLSS headers
    #include <nvsdk_ngx/nvsdk_ngx_dlss.h>
    #include <nvsdk_ngx/nvsdk_ngx_helpers.h>
    #include <nvsdk_ngx/nvsdk_ngx_params.h>
    
    // Define this macro so the stub implementation can be conditionally excluded
    #define HAS_REAL_DLSS_SDK
#else
    // DLSS SDK not available, use stub headers
    #include "stub/nvsdk_ngx.h"
    #include "stub/nvsdk_ngx_dlss.h"
    #include "stub/nvsdk_ngx_params.h"
#endif

// This wrapper header includes the necessary NVIDIA Streamline SDK headers.
// Bindgen will process this file.

// Main Streamline header
#include "sl.h"

// Streamline DLSS specific header (if needed and not pulled in by sl.h)
#include "sl_dlss.h"

// Other common Streamline headers you might need, depending on features used:
// #include "sl_reflex.h"
// #include "sl_common.h"
// #include "sl_helpers.h"

#endif // DLSS_WRAPPER_H 