<?php

namespace App\Events;

use App\Models\Review;
use Illuminate\Broadcasting\InteractsWithSockets;
use Illuminate\Foundation\Events\Dispatchable;
use Illuminate\Queue\SerializesModels;

class ReviewSubmitted
{
    use Dispatchable, InteractsWithSockets, SerializesModels;

    /**
     * The review instance.
     *
     * @var \App\Models\Review
     */
    public $review;

    /**
     * Create a new event instance.
     */
    public function __construct(Review $review)
    {
        $this->review = $review;
    }
} 