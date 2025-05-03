<?php

namespace App\Listeners;

use App\Events\FeedbackSubmitted;
use Illuminate\Contracts\Queue\ShouldQueue;
use Illuminate\Support\Facades\Log;

class LogFeedbackSubmission implements ShouldQueue
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
    public $queue = 'logging';

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
    public function handle(FeedbackSubmitted $event): void
    {
        $feedback = $event->feedback;
        $type = $event->type;

        Log::channel('feedback')->info("New $type feedback submitted", [
            'id' => $feedback->id,
            'type' => $type,
            'user_uuid' => $feedback->user_uuid,
            'created_at' => $feedback->created_at,
        ]);
    }
}
