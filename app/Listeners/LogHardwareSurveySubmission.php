<?php

namespace App\Listeners;

use App\Events\HardwareSurveySubmitted;
use Illuminate\Contracts\Queue\ShouldQueue;
use Illuminate\Queue\InteractsWithQueue;
use Illuminate\Support\Facades\Log;

class LogHardwareSurveySubmission implements ShouldQueue
{
    use InteractsWithQueue;

    /**
     * Create the event listener.
     */
    public function __construct()
    {
        //
    }

    /**
     * Handle the event.
     */
    public function handle(HardwareSurveySubmitted $event): void
    {
        Log::channel('feedback')->info('Hardware Survey submitted:', [
            'id' => $event->hardwareSurvey->id,
            'gpu' => $event->hardwareSurvey->gpu,
            'os' => $event->hardwareSurvey->os,
            'user_uuid' => $event->hardwareSurvey->user_uuid,
            'timestamp' => now()->toIso8601String(),
        ]);
    }
} 