<?php

namespace App\Listeners;

use App\Events\ReviewSubmitted;
use Illuminate\Contracts\Queue\ShouldQueue;
use Illuminate\Queue\InteractsWithQueue;
use Illuminate\Support\Facades\Log;

class LogReviewSubmission implements ShouldQueue
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
    public function handle(ReviewSubmitted $event): void
    {
        Log::channel('feedback')->info('Review submitted:', [
            'id' => $event->review->id,
            'rating' => $event->review->rating,
            'user_uuid' => $event->review->user_uuid,
            'timestamp' => now()->toIso8601String(),
        ]);
    }
} 