<?php

namespace App\Observers;

use App\Models\BugReport;

class BugReportObserver
{
    /**
     * Handle the BugReport "created" event.
     */
    public function created(BugReport $bugReport): void
    {
        //
    }

    /**
     * Handle the BugReport "updated" event.
     */
    public function updated(BugReport $bugReport): void
    {
        //
    }

    /**
     * Handle the BugReport "deleted" event.
     */
    public function deleted(BugReport $bugReport): void
    {
        //
    }

    /**
     * Handle the BugReport "restored" event.
     */
    public function restored(BugReport $bugReport): void
    {
        //
    }

    /**
     * Handle the BugReport "force deleted" event.
     */
    public function forceDeleted(BugReport $bugReport): void
    {
        //
    }
}
