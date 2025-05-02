<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;

class BugReport extends Model
{
    use HasFactory;

    protected $fillable = [
        'description',
        'log_path',
        'severity',
        'system_info',
        'user_uuid',
    ];

    protected $casts = [
        'system_info' => 'array', // Cast JSON column to array
    ];

    /**
     * Get the user that owns the bug report (optional).
     */
    public function user(): BelongsTo
    {
        return $this->belongsTo(User::class, 'user_uuid', 'uuid');
    }
} 