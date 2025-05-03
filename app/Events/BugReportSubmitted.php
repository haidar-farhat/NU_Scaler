<?php

namespace App\Events;

use App\Models\BugReport;
use Illuminate\Broadcasting\InteractsWithSockets;
use Illuminate\Foundation\Events\Dispatchable;
use Illuminate\Queue\SerializesModels;

class BugReportSubmitted
{
    use Dispatchable, InteractsWithSockets, SerializesModels;

    /**
     * The bug report instance.
     *
     * @var \App\Models\BugReport
     */
    public $bugReport;

    /**
     * Create a new event instance.
     */
    public function __construct(BugReport $bugReport)
    {
        $this->bugReport = $bugReport;
    }
} 