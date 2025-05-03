<?php

namespace App\Observers;

use App\Events\BugReportSubmitted;
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
     * Dispatch event for the submission.
     */
    public function created(BugReport $bugReport): void
    {
        // Dispatch event for the bug report submission
        event(new BugReportSubmitted($bugReport));
    }

    // Other observer methods
} 