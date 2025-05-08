<?php

namespace Tests\Feature\Api;

use App\Models\User;
use App\Models\Webhook;
use App\Models\WebhookLog;
use App\Services\WebhookService;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Illuminate\Foundation\Testing\WithFaker;
use Laravel\Sanctum\Sanctum;
use Tests\TestCase;
use Mockery;

class WebhookApiTest extends TestCase
{
    use RefreshDatabase, WithFaker;

    /**
     * Test webhook creation.
     *
     * @return void
     */
    public function test_authenticated_user_can_create_webhook(): void
    {
        $user = User::factory()->create();
        Sanctum::actingAs($user);

        $webhookData = [
            'name' => 'Test Webhook',
            'url' => 'https://example.com/webhook',
            'description' => 'Test webhook for events',
            'events' => ['feedback.review.created', 'feedback.bug.created'],
            'headers' => [
                'X-Custom-Header' => 'Custom Value',
            ],
        ];

        $response = $this->postJson('/api/v1/webhooks', $webhookData);

        $response->assertStatus(201)
            ->assertJsonStructure([
                'message',
                'data' => [
                    'id',
                    'name',
                    'url',
                    'description',
                    'events',
                    'headers',
                    'is_active',
                    'user_id',
                    'created_at',
                    'updated_at',
                ],
            ]);

        $this->assertDatabaseHas('webhooks', [
            'name' => 'Test Webhook',
            'url' => 'https://example.com/webhook',
            'user_id' => $user->id,
        ]);
    }

    /**
     * Test webhook validation.
     *
     * @return void
     */
    public function test_webhook_validation(): void
    {
        $user = User::factory()->create();
        Sanctum::actingAs($user);

        $response = $this->postJson('/api/v1/webhooks', [
            'name' => '', // Empty name
            'url' => 'not-a-valid-url', // Invalid URL
            'events' => 'not-an-array', // Not an array
        ]);

        $response->assertStatus(422)
            ->assertJsonValidationErrors(['name', 'url', 'events']);
    }

    /**
     * Test webhook listing.
     *
     * @return void
     */
    public function test_user_can_view_their_webhooks(): void
    {
        $user = User::factory()->create();
        Sanctum::actingAs($user);

        // Create some webhooks for this user
        Webhook::factory()->count(3)->create([
            'user_id' => $user->id,
        ]);

        // Create webhooks for another user (shouldn't be visible)
        Webhook::factory()->count(2)->create();

        $response = $this->getJson('/api/v1/webhooks');

        $response->assertStatus(200)
            ->assertJsonStructure([
                'data',
                'current_page',
                'total',
                'per_page',
            ]);

        // Should only see their own webhooks
        $this->assertEquals(3, $response->json('total'));
    }

    /**
     * Test webhook update.
     *
     * @return void
     */
    public function test_user_can_update_webhook(): void
    {
        $user = User::factory()->create();
        Sanctum::actingAs($user);

        $webhook = Webhook::factory()->create([
            'user_id' => $user->id,
        ]);

        $updateData = [
            'name' => 'Updated Webhook Name',
            'is_active' => false,
        ];

        $response = $this->putJson("/api/v1/webhooks/{$webhook->id}", $updateData);

        $response->assertStatus(200)
            ->assertJson([
                'message' => 'Webhook updated successfully',
                'data' => [
                    'name' => 'Updated Webhook Name',
                    'is_active' => false,
                ],
            ]);

        $this->assertDatabaseHas('webhooks', [
            'id' => $webhook->id,
            'name' => 'Updated Webhook Name',
            'is_active' => false,
        ]);
    }

    /**
     * Test webhook deletion.
     *
     * @return void
     */
    public function test_user_can_delete_webhook(): void
    {
        $user = User::factory()->create();
        Sanctum::actingAs($user);

        $webhook = Webhook::factory()->create([
            'user_id' => $user->id,
        ]);

        $response = $this->deleteJson("/api/v1/webhooks/{$webhook->id}");

        $response->assertStatus(200)
            ->assertJson([
                'message' => 'Webhook deleted successfully',
            ]);

        $this->assertDatabaseMissing('webhooks', [
            'id' => $webhook->id,
        ]);
    }

    /**
     * Test user cannot access another user's webhook.
     *
     * @return void
     */
    public function test_user_cannot_access_another_users_webhook(): void
    {
        $user1 = User::factory()->create();
        $user2 = User::factory()->create();

        Sanctum::actingAs($user1);

        $webhook = Webhook::factory()->create([
            'user_id' => $user2->id,
        ]);

        // Try to access
        $response = $this->getJson("/api/v1/webhooks/{$webhook->id}");
        $response->assertStatus(403);

        // Try to update
        $response = $this->putJson("/api/v1/webhooks/{$webhook->id}", ['name' => 'Hacked Webhook']);
        $response->assertStatus(403);

        // Try to delete
        $response = $this->deleteJson("/api/v1/webhooks/{$webhook->id}");
        $response->assertStatus(403);
    }

    /**
     * Test webhook regenerate secret.
     *
     * @return void
     */
    public function test_user_can_regenerate_webhook_secret(): void
    {
        $user = User::factory()->create();
        Sanctum::actingAs($user);

        $webhook = Webhook::factory()->create([
            'user_id' => $user->id,
            'secret' => 'old-secret',
        ]);

        $response = $this->postJson("/api/v1/webhooks/{$webhook->id}/regenerate-secret");

        $response->assertStatus(200)
            ->assertJsonStructure([
                'message',
                'data' => [
                    'secret',
                ],
            ]);

        // Verify the secret was changed
        $this->assertNotEquals('old-secret', Webhook::find($webhook->id)->secret);
    }

    /**
     * Test webhook test endpoint.
     *
     * @return void
     */
    public function test_user_can_test_webhook(): void
    {
        // Mock the WebhookService
        $mockService = Mockery::mock(WebhookService::class);
        $mockService->shouldReceive('sendWebhook')->once()->andReturn(true);
        $this->app->instance(WebhookService::class, $mockService);

        $user = User::factory()->create();
        Sanctum::actingAs($user);

        $webhook = Webhook::factory()->create([
            'user_id' => $user->id,
        ]);

        $response = $this->postJson("/api/v1/webhooks/{$webhook->id}/test");

        $response->assertStatus(200)
            ->assertJson([
                'message' => 'Webhook test sent successfully',
                'success' => true,
            ]);
    }

    /**
     * Test webhook retry endpoint.
     *
     * @return void
     */
    public function test_user_can_retry_webhook_delivery(): void
    {
        // Mock the WebhookService
        $mockService = Mockery::mock(WebhookService::class);
        $mockService->shouldReceive('retryWebhook')->once()->andReturn(true);
        $this->app->instance(WebhookService::class, $mockService);

        $user = User::factory()->create();
        Sanctum::actingAs($user);

        $webhook = Webhook::factory()->create([
            'user_id' => $user->id,
        ]);

        $log = WebhookLog::factory()->create([
            'webhook_id' => $webhook->id,
            'success' => false,
        ]);

        $response = $this->postJson("/api/v1/webhooks/logs/{$log->id}/retry");

        $response->assertStatus(200)
            ->assertJson([
                'message' => 'Webhook delivery retried successfully',
                'success' => true,
            ]);
    }
}
