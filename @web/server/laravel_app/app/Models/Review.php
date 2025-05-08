<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Builder;

class Review extends Model
{
    use HasFactory;

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'user_id',
        'rating',
        'comment',
        'name',
        'email',
        'user_uuid',
    ];

    /**
     * Scope a query to only include reviews of a given rating.
     *
     * @param  Builder $query
     * @param  int $rating
     * @return Builder
     */
    public function scopeRating(Builder $query, int $rating): Builder
    {
        return $query->where('rating', $rating);
    }

    /**
     * Apply filters to the query.
     *
     * @param Builder $query
     * @param array $filters
     * @return Builder
     */
    public function scopeFilter(Builder $query, array $filters): Builder
    {
        if (isset($filters['rating']) && is_numeric($filters['rating'])) {
             $query->rating((int)$filters['rating']);
        }

        // Add other filters here (e.g., date range)
        // if (isset($filters['start_date'])) {
        //     $query->whereDate('created_at', '>=', $filters['start_date']);
        // }
        // if (isset($filters['end_date'])) {
        //     $query->whereDate('created_at', '<=', $filters['end_date']);
        // }

        return $query;
    }
}
