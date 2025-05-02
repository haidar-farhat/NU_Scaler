<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;

class HardwareSurvey extends Model
{
    use HasFactory;

    protected $fillable = [
        'cpu',
        'gpu',
        'ram_gb',
        'os',
        'resolution',
        'user_uuid',
    ];

    /**
     * Get the user that owns the hardware survey (optional).
     */
    public function user(): BelongsTo
    {
        return $this->belongsTo(User::class, 'user_uuid', 'uuid');
    }
} 