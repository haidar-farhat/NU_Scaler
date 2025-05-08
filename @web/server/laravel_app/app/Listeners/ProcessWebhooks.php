<?php

namespace App\Listeners;

use App\Events\FeedbackSubmitted;
use App\Services\WebhookService;
use Illuminate\Contracts\Queue\ShouldQueue;
use Illuminate\Queue\InteractsWithQueue;

class ProcessWebhooks implements ShouldQueue
{
    /**
     * The name of the connection the job should be sent to.
     *
     * @var string|null
     */
    public $connection = 'redis';

    /**
     * The name of the queue the job should be sent to.
     *
     * @var string|null
     */
    public $queue = 'webhooks';

    /**
     * The webhook service instance.
     *
     * @var \App\Services\WebhookService
     */
    protected $webhookService;

    /**
     * Create the event listener.
     */
    public function __construct(WebhookService $webhookService)
    {
        $this->webhookService = $webhookService;
    }

    /**
     * Handle the event.
     */
    public function handle(FeedbackSubmitted $event): void
    {
        $feedback = $event->feedback;
        $type = $event->type;

        // Map event type to webhook event
        $webhookEvent = match($type) {
            'review' => 'feedback.review.created',
            'bug-report' => 'feedback.bug.created',
            'hardware-survey' => 'feedback.hardware.created',
            default => 'feedback.created',
        };

        // Create payload for the webhooks
        $payload = $this->createPayload($feedback, $type);

        // Dispatch webhooks
        $this->webhookService->dispatchEvent($webhookEvent, $payload);
    }

    /**
     * Create the webhook payload.
     *
     * @param mixed $feedback
     * @param string $type
     * @return array
     */
    protected function createPayload($feedback, string $type): array
    {
        $basePayload = [
            'id' => $feedback->id,
            'type' => $type,
            'created_at' => $feedback->created_at->toIso8601String(),
            'updated_at' => $feedback->updated_at->toIso8601String(),
        ];

        $specificPayload = match($type) {
            'review' => [
                'rating' => $feedback->rating,
                'comment' => $feedback->comment,
            ],
            'bug-report' => [
                'description' => $feedback->description,
                'category' => $feedback->category,
                'severity' => $feedback->severity,
                'system_info' => $feedback->system_info,
            ],
            'hardware-survey' => [
                'cpu_model' => $feedback->cpu_model,
                'gpu_model' => $feedback->gpu_model,
                'ram_size' => $feedback->ram_size,
                'os' => $feedback->os,
                'resolution' => $feedback->resolution,
            ],
            default => [],
        };

        return array_merge($basePayload, $specificPayload);
    }
}
