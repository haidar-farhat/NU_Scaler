<?php

namespace App\Observers;

use App\Events\HardwareSurveySubmitted;
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
     * Dispatch event for the submission.
     */
    public function created(HardwareSurvey $hardwareSurvey): void
    {
        // Dispatch event for the hardware survey submission
        event(new HardwareSurveySubmitted($hardwareSurvey));
    }

    // Other observer methods
} 