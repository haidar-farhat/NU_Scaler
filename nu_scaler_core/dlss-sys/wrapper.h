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

// This wrapper header includes the necessary NVIDIA DLSS SDK headers.
// Bindgen will process this file.

// Replace with the actual main header file from the DLSS SDK
// For example, if the main header is nvsdk_ngx.h:
#include "nvsdk_ngx.h"

// You might need to include other specific headers if the main one doesn't cover everything
// #include "nvsdk_ngx_defs.h"
// #include "nvsdk_ngx_params.h"
// #include "nvsdk_ngx_helpers.h"

#endif // DLSS_WRAPPER_H 