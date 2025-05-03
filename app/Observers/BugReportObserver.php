<?php

namespace App\Observers;

use App\Models\BugReport;
use Illuminate\Support\Facades\Log;

class BugReportObserver
{
    /**
     * Handle the BugReport "creating" event.
     * Sanitize text inputs.
     */
    public function creating(BugReport $bugReport): void
    {
        $bugReport->description = trim($bugReport->description);
        // Consider further sanitization for log_path if needed
    }

    /**
     * Handle the BugReport "created" event.
     * Log the submission.
     */
    public function created(BugReport $bugReport): void
    {
        Log::channel('feedback')->info('Bug Report submitted:', [
            'id' => $bugReport->id,
            'severity' => $bugReport->severity,
            'user_uuid' => $bugReport->user_uuid,
        ]);
    }

    // Other observer methods
} 