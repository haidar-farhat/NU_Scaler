/*
 * NVIDIA NGX Parameters Stub Implementation
 * This is a minimal stub implementation to allow compilation without the actual SDK
 */
#ifndef NVSDK_NGX_PARAMS_STUB_H
#define NVSDK_NGX_PARAMS_STUB_H

#include "nvsdk_ngx.h"

#ifdef __cplusplus
extern "C" {
#endif

// Parameter IDs for DLSS and other NVIDIA features
typedef enum {
    // Common parameters
    NVSDK_NGX_Parameter_Width = 0,
    NVSDK_NGX_Parameter_Height = 1,
    NVSDK_NGX_Parameter_Scale = 2,
    NVSDK_NGX_Parameter_Sharpness = 3,
    
    // DLSS specific parameters
    NVSDK_NGX_Parameter_DLSS_Feature_Create_Flags = 100,
    NVSDK_NGX_Parameter_DLSS_Input_Color = 101,
    NVSDK_NGX_Parameter_DLSS_Input_Depth = 102,
    NVSDK_NGX_Parameter_DLSS_Input_MotionVectors = 103,
    NVSDK_NGX_Parameter_DLSS_Output_Color = 104,
    NVSDK_NGX_Parameter_DLSS_Render_Width = 105,
    NVSDK_NGX_Parameter_DLSS_Render_Height = 106,
    NVSDK_NGX_Parameter_DLSS_Quality = 107,
    
    // Other parameters
    NVSDK_NGX_Parameter_ResourceAllocCallback = 1000,
    NVSDK_NGX_Parameter_ResourceReleaseCallback = 1001,
} NVSDK_NGX_Parameter_Type;

// Parameter type and value setting functions
static inline NVSDK_NGX_Result NVSDK_NGX_Parameter_GetI(NVSDK_NGX_Parameter_t InParams, NVSDK_NGX_Parameter_Type InType, unsigned int *OutValue) {
    *OutValue = 0;
    return NVSDK_NGX_Result_Fail;
}

static inline NVSDK_NGX_Result NVSDK_NGX_Parameter_GetF(NVSDK_NGX_Parameter_t InParams, NVSDK_NGX_Parameter_Type InType, float *OutValue) {
    *OutValue = 0.0f;
    return NVSDK_NGX_Result_Fail;
}

static inline NVSDK_NGX_Result NVSDK_NGX_Parameter_GetVoidPointer(NVSDK_NGX_Parameter_t InParams, NVSDK_NGX_Parameter_Type InType, void **OutValue) {
    *OutValue = nullptr;
    return NVSDK_NGX_Result_Fail;
}

static inline NVSDK_NGX_Result NVSDK_NGX_Parameter_SetI(NVSDK_NGX_Parameter_t InParams, NVSDK_NGX_Parameter_Type InType, unsigned int InValue) {
    return NVSDK_NGX_Result_Success;
}

static inline NVSDK_NGX_Result NVSDK_NGX_Parameter_SetF(NVSDK_NGX_Parameter_t InParams, NVSDK_NGX_Parameter_Type InType, float InValue) {
    return NVSDK_NGX_Result_Success;
}

static inline NVSDK_NGX_Result NVSDK_NGX_Parameter_SetVoidPointer(NVSDK_NGX_Parameter_t InParams, NVSDK_NGX_Parameter_Type InType, void *InValue) {
    return NVSDK_NGX_Result_Success;
}

#ifdef __cplusplus
}
#endif

#endif // NVSDK_NGX_PARAMS_STUB_H 