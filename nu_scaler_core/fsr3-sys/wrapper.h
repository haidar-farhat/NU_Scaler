// FSR 3.0 SDK wrapper header
#ifndef FSR3_WRAPPER_H
#define FSR3_WRAPPER_H

// Standard includes
#include <stdint.h>
#include <stdbool.h>

// Try to include FSR 3.0 SDK headers
#if __has_include(<ffx_fsr3.h>)
    // Main FSR 3.0 header
    #include <ffx_fsr3.h>
    #include <ffx_fsr3_api.h>
    
    // FSR 3.0 specific components
    #include <ffx_fsr3_upscaler.h>
    #include <ffx_fsr3_frameinterpolation.h>
    
    // Define this macro so the stub implementation can be conditionally excluded
    #define HAS_REAL_FSR3_SDK
#else
    // FSR 3.0 SDK not available, use stub headers
    #include "stub/ffx_fsr3.h"
    #include "stub/ffx_fsr3_upscaler.h"
    #include "stub/ffx_fsr3_frameinterpolation.h"
#endif

#endif // FSR3_WRAPPER_H 