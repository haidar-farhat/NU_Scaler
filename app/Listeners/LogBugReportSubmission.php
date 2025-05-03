<?php

namespace App\Listeners;

use App\Events\BugReportSubmitted;
use Illuminate\Contracts\Queue\ShouldQueue;
use Illuminate\Queue\InteractsWithQueue;
use Illuminate\Support\Facades\Log;

class LogBugReportSubmission implements ShouldQueue
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
    public function handle(BugReportSubmitted $event): void
    {
        Log::channel('feedback')->info('Bug Report submitted:', [
            'id' => $event->bugReport->id,
            'severity' => $event->bugReport->severity,
            'user_uuid' => $event->bugReport->user_uuid,
            'timestamp' => now()->toIso8601String(),
        ]);
        
        // If severity is high or critical, also log to a separate channel for urgent issues
        if (in_array($event->bugReport->severity, ['high', 'critical'])) {
            Log::channel('daily')->warning('High priority bug report received', [
                'id' => $event->bugReport->id,
                'severity' => $event->bugReport->severity,
                'description' => substr($event->bugReport->description, 0, 100) . '...',
            ]);
        }
    }
} 