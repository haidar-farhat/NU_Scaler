<?php

namespace App\Listeners;

use App\Events\BugReportSubmitted;
use App\Models\User;
use App\Notifications\CriticalBugReported;
use Illuminate\Contracts\Queue\ShouldQueue;
use Illuminate\Queue\InteractsWithQueue;
use Illuminate\Support\Facades\Notification;

class NotifyAboutCriticalBugReport implements ShouldQueue
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
        // Only notify for critical severity bug reports
        if ($event->bugReport->severity === 'critical') {
            // Get all admin users to notify
            $admins = User::where('is_admin', true)->get();
            
            // Send notification to all admins
            Notification::send($admins, new CriticalBugReported($event->bugReport));
        }
    }
} 