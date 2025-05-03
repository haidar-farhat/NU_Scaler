<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Builder;

class HardwareSurvey extends Model
{
    use HasFactory;

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'cpu_model',
        'gpu_model',
        'ram_size',
        'os',
        'resolution',
        'monitor_refresh_rate',
        'additional_info',
        'user_uuid',
    ];

    /**
     * Scope a query based on GPU name (case-insensitive partial match).
     *
     * @param  Builder $query
     * @param  string $gpuName
     * @return Builder
     */
    public function scopeGpuContains(Builder $query, string $gpuName): Builder
    {
        return $query->where('gpu_model', 'LIKE', '%'.$gpuName.'%');
    }

    /**
     * Scope a query based on OS name (case-insensitive partial match).
     *
     * @param  Builder $query
     * @param  string $osName
     * @return Builder
     */
    public function scopeOsContains(Builder $query, string $osName): Builder
    {
        return $query->where('os', 'LIKE', '%'.$osName.'%');
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
        if (!empty($filters['gpu'])) {
             $query->gpuContains($filters['gpu']);
        }

        if (!empty($filters['os'])) {
             $query->osContains($filters['os']);
        }

        // Add other filters (e.g., RAM range)

        return $query;
    }
}
