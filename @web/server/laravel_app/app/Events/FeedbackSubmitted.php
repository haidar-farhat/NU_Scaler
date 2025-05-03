<?php

namespace App\Events;

use Illuminate\Broadcasting\Channel;
use Illuminate\Broadcasting\InteractsWithSockets;
use Illuminate\Broadcasting\PrivateChannel;
use Illuminate\Contracts\Broadcasting\ShouldBroadcast;
use Illuminate\Foundation\Events\Dispatchable;
use Illuminate\Queue\SerializesModels;

class FeedbackSubmitted
{
    use Dispatchable, InteractsWithSockets, SerializesModels;

    /**
     * The feedback instance.
     *
     * @var mixed
     */
    public $feedback;

    /**
     * The feedback type.
     *
     * @var string
     */
    public $type;

    /**
     * Create a new event instance.
     *
     * @param mixed $feedback
     * @param string $type
     * @return void
     */
    public function __construct($feedback, string $type)
    {
        $this->feedback = $feedback;
        $this->type = $type;
    }

    /**
     * Get the channels the event should broadcast on.
     *
     * @return array<int, \Illuminate\Broadcasting\Channel>
     */
    public function broadcastOn(): array
    {
        return [
            new PrivateChannel('feedback'),
        ];
    }
}
