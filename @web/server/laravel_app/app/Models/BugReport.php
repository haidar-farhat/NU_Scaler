<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Builder;

class BugReport extends Model
{
    use HasFactory;

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'user_id',
        'description',
        'category',
        'severity',
        'steps_to_reproduce',
        'system_info',
        'user_uuid',
    ];

    /**
     * The attributes that should be cast.
     *
     * @var array
     */
    protected $casts = [
        'system_info' => 'json',
    ];

    /**
     * Scope a query to only include bug reports of a given severity.
     *
     * @param  Builder $query
     * @param  string $severity
     * @return Builder
     */
    public function scopeSeverity(Builder $query, string $severity): Builder
    {
        return $query->where('severity', $severity);
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
        if (!empty($filters['severity'])) {
             $query->severity($filters['severity']);
        }

        // Add other filters (e.g., date range)

        return $query;
    }
}
