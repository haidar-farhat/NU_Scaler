<?php

namespace App\Events;

use App\Models\HardwareSurvey;
use Illuminate\Broadcasting\InteractsWithSockets;
use Illuminate\Foundation\Events\Dispatchable;
use Illuminate\Queue\SerializesModels;

class HardwareSurveySubmitted
{
    use Dispatchable, InteractsWithSockets, SerializesModels;

    /**
     * The hardware survey instance.
     *
     * @var \App\Models\HardwareSurvey
     */
    public $hardwareSurvey;

    /**
     * Create a new event instance.
     */
    public function __construct(HardwareSurvey $hardwareSurvey)
    {
        $this->hardwareSurvey = $hardwareSurvey;
    }
} 