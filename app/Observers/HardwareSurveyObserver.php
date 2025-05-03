<?php

namespace App\Observers;

use App\Models\HardwareSurvey;
use Illuminate\Support\Facades\Log;

class HardwareSurveyObserver
{
    /**
     * Handle the HardwareSurvey "creating" event.
     * Sanitize text inputs.
     */
    public function creating(HardwareSurvey $hardwareSurvey): void
    {
        $hardwareSurvey->cpu = isset($hardwareSurvey->cpu) ? trim($hardwareSurvey->cpu) : null;
        $hardwareSurvey->gpu = isset($hardwareSurvey->gpu) ? trim($hardwareSurvey->gpu) : null;
        $hardwareSurvey->os = isset($hardwareSurvey->os) ? trim($hardwareSurvey->os) : null;
        $hardwareSurvey->resolution = isset($hardwareSurvey->resolution) ? trim($hardwareSurvey->resolution) : null;
    }

    /**
     * Handle the HardwareSurvey "created" event.
     * Log the submission.
     */
    public function created(HardwareSurvey $hardwareSurvey): void
    {
        Log::channel('feedback')->info('Hardware Survey submitted:', [
            'id' => $hardwareSurvey->id,
            'gpu' => $hardwareSurvey->gpu, // Log GPU for quick reference
            'user_uuid' => $hardwareSurvey->user_uuid,
        ]);
    }

    // Other observer methods
} 