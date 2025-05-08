<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;

class WebhookLog extends Model
{
    use HasFactory;

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'webhook_id',
        'event_type',
        'payload',
        'status_code',
        'response',
        'error',
        'success',
    ];

    /**
     * The attributes that should be cast.
     *
     * @var array<string, string>
     */
    protected $casts = [
        'payload' => 'json',
        'success' => 'boolean',
        'status_code' => 'integer',
    ];

    /**
     * Get the webhook that owns the log.
     */
    public function webhook()
    {
        return $this->belongsTo(Webhook::class);
    }
}
