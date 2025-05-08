<?php

namespace App\Observers;

use App\Models\HardwareSurvey;

class HardwareSurveyObserver
{
    /**
     * Handle the HardwareSurvey "created" event.
     */
    public function created(HardwareSurvey $hardwareSurvey): void
    {
        //
    }

    /**
     * Handle the HardwareSurvey "updated" event.
     */
    public function updated(HardwareSurvey $hardwareSurvey): void
    {
        //
    }

    /**
     * Handle the HardwareSurvey "deleted" event.
     */
    public function deleted(HardwareSurvey $hardwareSurvey): void
    {
        //
    }

    /**
     * Handle the HardwareSurvey "restored" event.
     */
    public function restored(HardwareSurvey $hardwareSurvey): void
    {
        //
    }

    /**
     * Handle the HardwareSurvey "force deleted" event.
     */
    public function forceDeleted(HardwareSurvey $hardwareSurvey): void
    {
        //
    }
}
