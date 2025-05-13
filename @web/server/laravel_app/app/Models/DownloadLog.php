<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;

class DownloadLog extends Model
{
    use HasFactory;

    /**
     * We only need created_at, disable updated_at.
     */
    public $timestamps = false; // Disable updated_at
    const CREATED_AT = 'created_at'; // Explicitly define created_at if needed

    /**
     * The attributes that are mass assignable.
     *
     * @var array<int, string>
     */
    protected $fillable = [
        'user_id',
        'ip_address',
        'platform',
        'downloaded',
        // created_at is handled automatically
    ];

    /**
     * The attributes that should be cast.
     *
     * @var array<string, string>
     */
    protected $casts = [
        'downloaded' => 'boolean',
    ];

    /**
     * Get the user that owns the download log.
     */
    public function user()
    {
        return $this->belongsTo(User::class);
    }
}
