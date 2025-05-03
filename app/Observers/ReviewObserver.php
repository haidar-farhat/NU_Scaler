<?php

namespace App\Observers;

use App\Models\Review;
use Illuminate\Support\Facades\Log;

class ReviewObserver
{
    /**
     * Handle the Review "creating" event.
     * Sanitize text inputs before saving.
     */
    public function creating(Review $review): void
    {
        $review->comment = isset($review->comment) ? trim($review->comment) : null;
        $review->name = isset($review->name) ? trim($review->name) : null;
        $review->email = isset($review->email) ? trim($review->email) : null;
    }

    /**
     * Handle the Review "created" event.
     * Log the submission to the feedback channel.
     */
    public function created(Review $review): void
    {
        Log::channel('feedback')->info('Review submitted:', [
            'id' => $review->id,
            'rating' => $review->rating,
            'user_uuid' => $review->user_uuid,
        ]);
    }

    // Other observer methods (updated, deleted, etc.) can be added here if needed
} 