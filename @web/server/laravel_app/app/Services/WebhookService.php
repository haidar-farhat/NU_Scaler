<?php

namespace App\Services;

use App\Models\Webhook;
use App\Models\WebhookLog;
use Exception;
use Illuminate\Http\Client\RequestException;
use Illuminate\Support\Facades\Http;
use Illuminate\Support\Facades\Log;

class WebhookService
{
    /**
     * Dispatch an event to all registered webhooks.
     *
     * @param string $event
     * @param array $payload
     * @return void
     */
    public function dispatchEvent(string $event, array $payload): void
    {
        $webhooks = Webhook::where('is_active', true)
            ->get()
            ->filter(function($webhook) use ($event) {
                return $webhook->shouldTriggerFor($event);
            });

        foreach ($webhooks as $webhook) {
            dispatch(function() use ($webhook, $event, $payload) {
                $this->sendWebhook($webhook, $event, $payload);
            })->onQueue('webhooks');
        }
    }

    /**
     * Send a webhook to a specific URL.
     *
     * @param \App\Models\Webhook $webhook
     * @param string $event
     * @param array $payload
     * @return bool
     */
    public function sendWebhook(Webhook $webhook, string $event, array $payload): bool
    {
        // Create a log entry
        $log = WebhookLog::create([
            'webhook_id' => $webhook->id,
            'event_type' => $event,
            'payload' => $payload,
            'success' => false,
        ]);

        // Prepare headers
        $headers = array_merge(
            [
                'Content-Type' => 'application/json',
                'User-Agent' => 'Nu-Scaler-Webhook/1.0',
                'X-Nu-Scaler-Event' => $event,
                'X-Nu-Scaler-Delivery' => $log->id,
            ],
            $webhook->headers ?? []
        );

        // Add signature if secret is set
        if ($webhook->secret) {
            $signature = $this->generateSignature($webhook->secret, json_encode($payload));
            $headers['X-Nu-Scaler-Signature'] = $signature;
        }

        try {
            // Send the webhook
            $response = Http::withHeaders($headers)
                ->timeout(30)
                ->post($webhook->url, [
                    'event' => $event,
                    'payload' => $payload,
                ]);

            // Update log
            $log->update([
                'status_code' => $response->status(),
                'response' => $response->body(),
                'success' => $response->successful(),
            ]);

            // Update webhook
            if ($response->successful()) {
                $webhook->markAsTriggered();
                return true;
            } else {
                $webhook->incrementFailCount();
                return false;
            }

        } catch (RequestException $e) {
            // Log the error
            $log->update([
                'error' => $e->getMessage(),
                'success' => false,
            ]);

            $webhook->incrementFailCount();

            Log::error('Webhook delivery failed', [
                'webhook_id' => $webhook->id,
                'url' => $webhook->url,
                'event' => $event,
                'error' => $e->getMessage(),
            ]);

            return false;
        } catch (Exception $e) {
            // Log any other exceptions
            $log->update([
                'error' => $e->getMessage(),
                'success' => false,
            ]);

            $webhook->incrementFailCount();

            Log::error('Webhook delivery failed with exception', [
                'webhook_id' => $webhook->id,
                'url' => $webhook->url,
                'event' => $event,
                'error' => $e->getMessage(),
            ]);

            return false;
        }
    }

    /**
     * Generate a signature for the payload.
     *
     * @param string $secret
     * @param string $payload
     * @return string
     */
    protected function generateSignature(string $secret, string $payload): string
    {
        return hash_hmac('sha256', $payload, $secret);
    }

    /**
     * Retry a failed webhook delivery.
     *
     * @param \App\Models\WebhookLog $log
     * @return bool
     */
    public function retryWebhook(WebhookLog $log): bool
    {
        $webhook = $log->webhook;

        if (!$webhook || !$webhook->is_active) {
            return false;
        }

        return $this->sendWebhook(
            $webhook,
            $log->event_type,
            $log->payload
        );
    }
}
