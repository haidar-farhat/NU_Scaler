<?php

namespace App\Listeners;

use App\Events\FeedbackSubmitted;
use App\Services\AnalyticsService;
use Illuminate\Contracts\Queue\ShouldQueue;

class ProcessFeedbackAnalytics implements ShouldQueue
{
    /**
     * The name of the connection the job should be sent to.
     *
     * @var string|null
     */
    public $connection = 'redis';

    /**
     * The name of the queue the job should be sent to.
     *
     * @var string|null
     */
    public $queue = 'analytics';

    /**
     * The analytics service instance.
     *
     * @var \App\Services\AnalyticsService
     */
    protected $analyticsService;

    /**
     * Create the event listener.
     */
    public function __construct(AnalyticsService $analyticsService)
    {
        $this->analyticsService = $analyticsService;
    }

    /**
     * Handle the event.
     */
    public function handle(FeedbackSubmitted $event): void
    {
        $this->analyticsService->processFeedback($event->feedback, $event->type);
    }
}
