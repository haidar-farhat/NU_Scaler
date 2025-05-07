/*
 * AMD FSR 3.0 Stub Implementation
 * This is a minimal stub implementation to allow compilation without the actual SDK
 */
#ifndef FFX_FSR3_STUB_H
#define FFX_FSR3_STUB_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

// Define common types from FSR 3.0 SDK
typedef uint32_t FfxErrorCode;
typedef struct FfxDevice* FfxDevice;
typedef struct FfxCommandList* FfxCommandList;
typedef struct FfxResource* FfxResource;
typedef struct FfxInterface* FfxInterface;

// Return values
#define FFX_OK                      0
#define FFX_ERROR_INVALID_POINTER   1
#define FFX_ERROR_INVALID_ARGUMENT  2
#define FFX_ERROR_OUT_OF_MEMORY     3
#define FFX_ERROR_NOT_IMPLEMENTED   4
#define FFX_ERROR_NULL_DEVICE       5
#define FFX_ERROR_BACKEND_API_ERROR 6

// Version information
#define FFX_FSR3_API_VERSION       0x00010000u  // Version 1.0.0
#define FFX_FSR3_VERSION_MAJOR     3
#define FFX_FSR3_VERSION_MINOR     0
#define FFX_FSR3_VERSION_PATCH     0

// Quality settings (mirrors DLSS quality levels for consistency)
typedef enum FfxFsr3QualityMode {
    FFX_FSR3_QUALITY_MODE_QUALITY = 2,
    FFX_FSR3_QUALITY_MODE_BALANCED = 1,
    FFX_FSR3_QUALITY_MODE_PERFORMANCE = 0,
    FFX_FSR3_QUALITY_MODE_ULTRA_PERFORMANCE = 3,
    FFX_FSR3_QUALITY_MODE_ULTRA_QUALITY = 4
} FfxFsr3QualityMode;

// Basic structure for FSR 3 context
typedef struct FfxFsr3Context {
    int initialized;
    // Placeholder members for compatibility
    void* device;
    int width;
    int height;
    FfxFsr3QualityMode quality;
} FfxFsr3Context;

// Basic logging implementation
typedef enum FfxFsr3LogLevel {
    FFX_FSR3_LOG_LEVEL_DEBUG,
    FFX_FSR3_LOG_LEVEL_INFO,
    FFX_FSR3_LOG_LEVEL_WARNING,
    FFX_FSR3_LOG_LEVEL_ERROR
} FfxFsr3LogLevel;

typedef void (*FfxFsr3LogCallback)(FfxFsr3LogLevel level, const char* message);

// Helper functions for creating and destroying contexts
static inline FfxErrorCode ffxFsr3ContextCreate(FfxFsr3Context* context, FfxInterface* interface) {
    if (!context) {
        return FFX_ERROR_INVALID_POINTER;
    }
    context->initialized = 1;
    context->device = NULL;
    context->width = 0;
    context->height = 0;
    context->quality = FFX_FSR3_QUALITY_MODE_QUALITY;
    return FFX_OK;
}

static inline FfxErrorCode ffxFsr3ContextDestroy(FfxFsr3Context* context) {
    if (!context) {
        return FFX_ERROR_INVALID_POINTER;
    }
    context->initialized = 0;
    return FFX_OK;
}

// FSR 3 capabilities check
static inline bool ffxFsr3IsAvailable() {
    // Always return true in the stub for testing
    return true;
}

#ifdef __cplusplus
}
#endif

#endif // FFX_FSR3_STUB_H 