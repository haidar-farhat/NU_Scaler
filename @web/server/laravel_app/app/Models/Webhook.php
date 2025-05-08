<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Support\Str;

class Webhook extends Model
{
    use HasFactory;

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'name',
        'url',
        'description',
        'is_active',
        'events',
        'secret',
        'user_id',
        'headers',
        'last_triggered_at',
        'fails_count',
    ];

    /**
     * The attributes that should be cast.
     *
     * @var array<string, string>
     */
    protected $casts = [
        'is_active' => 'boolean',
        'events' => 'json',
        'headers' => 'json',
        'last_triggered_at' => 'datetime',
        'fails_count' => 'integer',
    ];

    /**
     * The attributes that should be hidden for serialization.
     *
     * @var array<int, string>
     */
    protected $hidden = [
        'secret',
    ];

    /**
     * Generate a new webhook secret.
     *
     * @return string
     */
    public static function generateSecret(): string
    {
        return Str::random(40);
    }

    /**
     * Check if the webhook should be triggered for an event.
     *
     * @param string $event
     * @return bool
     */
    public function shouldTriggerFor(string $event): bool
    {
        // If webhook is not active, it should not be triggered
        if (!$this->is_active) {
            return false;
        }

        // If there are too many failures, disable the webhook
        if ($this->fails_count >= 10) {
            $this->update(['is_active' => false]);
            return false;
        }

        // Check if the webhook is subscribed to this event
        $events = $this->events;
        if (in_array('*', $events) || in_array($event, $events)) {
            return true;
        }

        return false;
    }

    /**
     * Set webhook as triggered successfully.
     *
     * @return void
     */
    public function markAsTriggered(): void
    {
        $this->update([
            'last_triggered_at' => now(),
            'fails_count' => 0,
        ]);
    }

    /**
     * Increment the failure count.
     *
     * @return void
     */
    public function incrementFailCount(): void
    {
        $this->increment('fails_count');

        // If there are too many failures, disable the webhook
        if ($this->fails_count >= 10) {
            $this->update(['is_active' => false]);
        }
    }

    /**
     * Get the user that owns the webhook.
     */
    public function user()
    {
        return $this->belongsTo(User::class);
    }
}
