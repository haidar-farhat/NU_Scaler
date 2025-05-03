<?php

namespace App\Http\Controllers\Api\V1;

use App\Http\Controllers\Controller;
use App\Models\Webhook;
use App\Models\WebhookLog;
use App\Services\WebhookService;
use Illuminate\Http\JsonResponse;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Validator;

class WebhookController extends Controller
{
    /**
     * The webhook service instance.
     *
     * @var \App\Services\WebhookService
     */
    protected $webhookService;

    /**
     * Create a new controller instance.
     *
     * @param \App\Services\WebhookService $webhookService
     * @return void
     */
    public function __construct(WebhookService $webhookService)
    {
        $this->webhookService = $webhookService;
    }

    /**
     * Display a listing of the resource.
     *
     * @param Request $request
     * @return JsonResponse
     */
    public function index(Request $request): JsonResponse
    {
        $webhooks = Webhook::where('user_id', $request->user()->id)
            ->latest()
            ->paginate();

        return response()->json($webhooks);
    }

    /**
     * Store a newly created resource in storage.
     *
     * @param Request $request
     * @return JsonResponse
     */
    public function store(Request $request): JsonResponse
    {
        $validator = Validator::make($request->all(), [
            'name' => 'required|string|max:255',
            'url' => 'required|url|max:1000',
            'description' => 'nullable|string|max:1000',
            'events' => 'required|array',
            'events.*' => 'string|in:feedback.review.created,feedback.bug.created,feedback.hardware.created,user.registered',
            'headers' => 'nullable|array',
            'headers.*' => 'string',
        ]);

        if ($validator->fails()) {
            return response()->json([
                'message' => 'Validation failed',
                'errors' => $validator->errors(),
            ], 422);
        }

        $webhook = new Webhook([
            'name' => $request->input('name'),
            'url' => $request->input('url'),
            'description' => $request->input('description'),
            'events' => $request->input('events'),
            'headers' => $request->input('headers', []),
            'secret' => Webhook::generateSecret(),
            'user_id' => $request->user()->id,
            'is_active' => true,
        ]);

        $webhook->save();

        return response()->json([
            'message' => 'Webhook created successfully',
            'data' => $webhook,
        ], 201);
    }

    /**
     * Display the specified resource.
     *
     * @param Webhook $webhook
     * @param Request $request
     * @return JsonResponse
     */
    public function show(Webhook $webhook, Request $request): JsonResponse
    {
        // Check if the webhook belongs to the user
        if ($webhook->user_id !== $request->user()->id) {
            return response()->json([
                'message' => 'Forbidden',
            ], 403);
        }

        return response()->json([
            'data' => $webhook,
        ]);
    }

    /**
     * Update the specified resource in storage.
     *
     * @param Request $request
     * @param Webhook $webhook
     * @return JsonResponse
     */
    public function update(Request $request, Webhook $webhook): JsonResponse
    {
        // Check if the webhook belongs to the user
        if ($webhook->user_id !== $request->user()->id) {
            return response()->json([
                'message' => 'Forbidden',
            ], 403);
        }

        $validator = Validator::make($request->all(), [
            'name' => 'sometimes|required|string|max:255',
            'url' => 'sometimes|required|url|max:1000',
            'description' => 'nullable|string|max:1000',
            'events' => 'sometimes|required|array',
            'events.*' => 'string|in:feedback.review.created,feedback.bug.created,feedback.hardware.created,user.registered',
            'headers' => 'nullable|array',
            'headers.*' => 'string',
            'is_active' => 'sometimes|boolean',
        ]);

        if ($validator->fails()) {
            return response()->json([
                'message' => 'Validation failed',
                'errors' => $validator->errors(),
            ], 422);
        }

        $webhook->fill($request->only([
            'name',
            'url',
            'description',
            'events',
            'headers',
            'is_active',
        ]));

        $webhook->save();

        return response()->json([
            'message' => 'Webhook updated successfully',
            'data' => $webhook,
        ]);
    }

    /**
     * Remove the specified resource from storage.
     *
     * @param Webhook $webhook
     * @param Request $request
     * @return JsonResponse
     */
    public function destroy(Webhook $webhook, Request $request): JsonResponse
    {
        // Check if the webhook belongs to the user
        if ($webhook->user_id !== $request->user()->id) {
            return response()->json([
                'message' => 'Forbidden',
            ], 403);
        }

        $webhook->delete();

        return response()->json([
            'message' => 'Webhook deleted successfully',
        ]);
    }

    /**
     * Get webhook logs.
     *
     * @param Webhook $webhook
     * @param Request $request
     * @return JsonResponse
     */
    public function logs(Webhook $webhook, Request $request): JsonResponse
    {
        // Check if the webhook belongs to the user
        if ($webhook->user_id !== $request->user()->id) {
            return response()->json([
                'message' => 'Forbidden',
            ], 403);
        }

        $logs = WebhookLog::where('webhook_id', $webhook->id)
            ->latest()
            ->paginate();

        return response()->json($logs);
    }

    /**
     * Regenerate the webhook secret.
     *
     * @param Webhook $webhook
     * @param Request $request
     * @return JsonResponse
     */
    public function regenerateSecret(Webhook $webhook, Request $request): JsonResponse
    {
        // Check if the webhook belongs to the user
        if ($webhook->user_id !== $request->user()->id) {
            return response()->json([
                'message' => 'Forbidden',
            ], 403);
        }

        $webhook->secret = Webhook::generateSecret();
        $webhook->save();

        return response()->json([
            'message' => 'Webhook secret regenerated successfully',
            'data' => [
                'secret' => $webhook->secret,
            ],
        ]);
    }

    /**
     * Test a webhook.
     *
     * @param Webhook $webhook
     * @param Request $request
     * @return JsonResponse
     */
    public function test(Webhook $webhook, Request $request): JsonResponse
    {
        // Check if the webhook belongs to the user
        if ($webhook->user_id !== $request->user()->id) {
            return response()->json([
                'message' => 'Forbidden',
            ], 403);
        }

        // Test the webhook with a ping event
        $success = $this->webhookService->sendWebhook($webhook, 'ping', [
            'message' => 'This is a test ping from Nu Scaler',
            'timestamp' => now()->toIso8601String(),
        ]);

        return response()->json([
            'message' => $success ? 'Webhook test sent successfully' : 'Webhook test failed',
            'success' => $success,
        ]);
    }

    /**
     * Retry a webhook log.
     *
     * @param WebhookLog $log
     * @param Request $request
     * @return JsonResponse
     */
    public function retry(WebhookLog $log, Request $request): JsonResponse
    {
        // Check if the webhook belongs to the user
        if ($log->webhook->user_id !== $request->user()->id) {
            return response()->json([
                'message' => 'Forbidden',
            ], 403);
        }

        $success = $this->webhookService->retryWebhook($log);

        return response()->json([
            'message' => $success ? 'Webhook delivery retried successfully' : 'Webhook retry failed',
            'success' => $success,
        ]);
    }
}
