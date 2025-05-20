<?php

namespace App\Http\Controllers\Api\V1;

use App\Http\Controllers\Controller;
use App\Models\Webhook;
use App\Models\WebhookLog;
use App\Services\WebhookService;
use Illuminate\Http\JsonResponse;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Validator;
use App\Http\Requests\Api\V1\StoreWebhookRequest;
use App\Http\Requests\Api\V1\UpdateWebhookRequest;
use App\Http\Responses\ApiResponse;
use App\Repositories\WebhookRepository;

class WebhookController extends Controller
{
    /**
     * The webhook service instance.
     *
     * @var \App\Services\WebhookService
     */
    protected $webhookService;

    /**
     * The webhook repository instance.
     *
     * @var \App\Repositories\WebhookRepository
     */
    protected $webhookRepository;

    /**
     * Create a new controller instance.
     *
     * @param \App\Services\WebhookService $webhookService
     * @param \App\Repositories\WebhookRepository $webhookRepository
     * @return void
     */
    public function __construct(WebhookService $webhookService, WebhookRepository $webhookRepository)
    {
        $this->webhookService = $webhookService;
        $this->webhookRepository = $webhookRepository;
    }

    /**
     * Display a listing of the resource.
     *
     * @param Request $request
     * @return JsonResponse
     */
    public function index(Request $request): JsonResponse
    {
        $webhooks = $this->webhookRepository->findByUser($request->user()->id);
        return ApiResponse::success('Webhooks fetched successfully', $webhooks);
    }

    /**
     * Store a newly created resource in storage.
     *
     * @param StoreWebhookRequest $request
     * @return JsonResponse
     */
    public function store(StoreWebhookRequest $request): JsonResponse
    {
        $webhook = $this->webhookService->create($request->validated(), $request->user());
        return ApiResponse::success('Webhook created successfully', $webhook, 201);
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
        if ($webhook->user_id !== $request->user()->id) {
            return ApiResponse::error('Forbidden', null, 403);
        }
        return ApiResponse::success('Webhook fetched successfully', $webhook);
    }

    /**
     * Update the specified resource in storage.
     *
     * @param UpdateWebhookRequest $request
     * @param Webhook $webhook
     * @return JsonResponse
     */
    public function update(UpdateWebhookRequest $request, Webhook $webhook): JsonResponse
    {
        if ($webhook->user_id !== $request->user()->id) {
            return ApiResponse::error('Forbidden', null, 403);
        }
        $webhook = $this->webhookService->update($webhook, $request->validated());
        return ApiResponse::success('Webhook updated successfully', $webhook);
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
        if ($webhook->user_id !== $request->user()->id) {
            return ApiResponse::error('Forbidden', null, 403);
        }
        $this->webhookService->delete($webhook);
        return ApiResponse::success('Webhook deleted successfully');
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
