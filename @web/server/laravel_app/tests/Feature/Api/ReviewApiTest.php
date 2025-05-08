<?php

namespace Tests\Feature\Api;

use App\Models\Review;
use App\Models\User;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Illuminate\Foundation\Testing\WithFaker;
use Laravel\Sanctum\Sanctum;
use Tests\TestCase;

class ReviewApiTest extends TestCase
{
    use RefreshDatabase, WithFaker;

    /**
     * Test that a review can be submitted by a guest.
     *
     * @return void
     */
    public function test_guest_can_submit_review(): void
    {
        $reviewData = [
            'rating' => 4,
            'comment' => 'This is a great product!',
            'name' => 'John Doe',
            'email' => 'john@example.com',
        ];

        $response = $this->postJson('/api/v1/feedback/reviews', $reviewData);

        $response->assertStatus(201)
            ->assertJsonStructure([
                'message',
                'data' => [
                    'id',
                    'rating',
                    'comment',
                    'name',
                    'email',
                    'created_at',
                    'updated_at',
                ],
            ]);

        $this->assertDatabaseHas('reviews', [
            'rating' => 4,
            'comment' => 'This is a great product!',
            'name' => 'John Doe',
            'email' => 'john@example.com',
        ]);
    }

    /**
     * Test that a review can be submitted by an authenticated user.
     *
     * @return void
     */
    public function test_authenticated_user_can_submit_review(): void
    {
        $user = User::factory()->create();
        Sanctum::actingAs($user);

        $reviewData = [
            'rating' => 5,
            'comment' => 'Works perfectly!',
        ];

        $response = $this->postJson('/api/v1/feedback/reviews', $reviewData);

        $response->assertStatus(201)
            ->assertJsonStructure([
                'message',
                'data' => [
                    'id',
                    'rating',
                    'comment',
                    'user_uuid',
                    'created_at',
                    'updated_at',
                ],
            ]);

        $this->assertDatabaseHas('reviews', [
            'rating' => 5,
            'comment' => 'Works perfectly!',
            'user_uuid' => $user->uuid,
        ]);
    }

    /**
     * Test that review submission requires valid data.
     *
     * @return void
     */
    public function test_review_validation(): void
    {
        $response = $this->postJson('/api/v1/feedback/reviews', [
            'rating' => 10, // Invalid rating (>5)
            'comment' => '', // Empty comment
        ]);

        $response->assertStatus(422)
            ->assertJsonValidationErrors(['rating', 'comment']);
    }

    /**
     * Test that admins can view all reviews.
     *
     * @return void
     */
    public function test_admin_can_view_all_reviews(): void
    {
        // Create an admin user
        $admin = User::factory()->create(['is_admin' => true]);
        Sanctum::actingAs($admin);

        // Create some reviews
        Review::factory()->count(5)->create();

        $response = $this->getJson('/api/admin/reviews');

        $response->assertStatus(200)
            ->assertJsonStructure([
                'data',
                'current_page',
                'total',
                'per_page',
            ]);
    }

    /**
     * Test that non-admins cannot access admin routes.
     *
     * @return void
     */
    public function test_non_admin_cannot_access_admin_reviews(): void
    {
        // Create a regular user
        $user = User::factory()->create(['is_admin' => false]);
        Sanctum::actingAs($user);

        // Create some reviews
        Review::factory()->count(5)->create();

        $response = $this->getJson('/api/admin/reviews');

        $response->assertStatus(403);
    }
}
