<?php

namespace App\Services;

use App\Models\Review;

class ReviewService
{
    public function create(array $data): Review
    {
        return Review::create($data);
    }
}
