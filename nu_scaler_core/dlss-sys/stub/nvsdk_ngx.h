/*
 * NVIDIA NGX SDK Stub Implementation
 * This is a minimal stub implementation to allow compilation without the actual SDK
 */
#ifndef NVSDK_NGX_STUB_H
#define NVSDK_NGX_STUB_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// Common type definitions
typedef uint32_t NVSDK_NGX_Result;
typedef struct NVSDK_NGX_Handle* NVSDK_NGX_Handle_t;
typedef struct NVSDK_NGX_Parameter* NVSDK_NGX_Parameter_t;

// Versions
#define NVSDK_NGX_VERSION_API_MAJOR 1
#define NVSDK_NGX_VERSION_API_MINOR 0

// NGX feature IDs
typedef enum {
    NVSDK_NGX_Feature_SuperSampling = 0,
    NVSDK_NGX_Feature_InPainting = 1,
    NVSDK_NGX_Feature_ImageSuperResolution = 2,
    NVSDK_NGX_Feature_SlowMotion = 3,
    NVSDK_NGX_Feature_VideoSuperResolution = 4,
    NVSDK_NGX_Feature_MAX
} NVSDK_NGX_Feature;

// Error codes
#define NVSDK_NGX_Result_Success                   0x1
#define NVSDK_NGX_Result_Fail                      0x0
#define NVSDK_NGX_Result_FAIL_FeatureNotSupported  0xBEEF0001
#define NVSDK_NGX_Result_FAIL_NotInitialized       0xBEEF0002
#define NVSDK_NGX_Result_FAIL_UnsupportedFormat    0xBEEF0003
#define NVSDK_NGX_Result_FAIL_OutOfMemory          0xBEEF0004

// Stub implementation for core functions
static inline NVSDK_NGX_Result NVSDK_NGX_Init(const char* InApplicationId, const char* InApplicationDataPath, void* InDevice) {
    return NVSDK_NGX_Result_FAIL_FeatureNotSupported;
}

static inline NVSDK_NGX_Result NVSDK_NGX_Shutdown() {
    return NVSDK_NGX_Result_Success;
}

static inline NVSDK_NGX_Result NVSDK_NGX_GetScratchBufferSize(NVSDK_NGX_Feature InFeatureId, const NVSDK_NGX_Parameter_t InParameters, size_t* OutSizeInBytes) {
    *OutSizeInBytes = 0;
    return NVSDK_NGX_Result_FAIL_FeatureNotSupported;
}

static inline NVSDK_NGX_Result NVSDK_NGX_CreateFeature(void* InCmdList, NVSDK_NGX_Feature InFeatureId, NVSDK_NGX_Parameter_t InParameters, NVSDK_NGX_Handle_t* OutHandle) {
    return NVSDK_NGX_Result_FAIL_FeatureNotSupported;
}

static inline NVSDK_NGX_Result NVSDK_NGX_Release(NVSDK_NGX_Handle_t InHandle) {
    return NVSDK_NGX_Result_Success;
}

static inline NVSDK_NGX_Result NVSDK_NGX_EvaluateFeature(void* InCmdList, NVSDK_NGX_Handle_t InHandle, NVSDK_NGX_Parameter_t InParameters) {
    return NVSDK_NGX_Result_FAIL_FeatureNotSupported;
}

static inline NVSDK_NGX_Result NVSDK_NGX_AllocateParameters(NVSDK_NGX_Parameter_t* OutParameters) {
    return NVSDK_NGX_Result_FAIL_FeatureNotSupported;
}

static inline NVSDK_NGX_Result NVSDK_NGX_DestroyParameters(NVSDK_NGX_Parameter_t InParameters) {
    return NVSDK_NGX_Result_Success;
}

#ifdef __cplusplus
}
#endif

#endif // NVSDK_NGX_STUB_H 