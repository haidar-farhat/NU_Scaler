<?php

namespace App\Observers;

use App\Events\ReviewSubmitted;
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
     * Dispatch event for the submission.
     */
    public function created(Review $review): void
    {
        // Dispatch event for the review submission
        event(new ReviewSubmitted($review));
    }

    // Other observer methods (updated, deleted, etc.) can be added here if needed
} 