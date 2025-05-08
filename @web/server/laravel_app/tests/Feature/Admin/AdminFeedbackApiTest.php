<?php

namespace Tests\Feature\Admin;

use Illuminate\Foundation\Testing\RefreshDatabase;
use Tests\TestCase;
use App\Models\User;
use App\Models\Review;
use Laravel\Sanctum\Sanctum;

class AdminFeedbackApiTest extends TestCase
{
    use RefreshDatabase;

    protected User $adminUser;

    protected function setUp(): void
    {
        parent::setUp();

        // Create and authenticate an admin user for these tests
        $this->adminUser = User::factory()->create(['is_admin' => true]);
        Sanctum::actingAs($this->adminUser, ['*']); // Give admin all abilities for simplicity
    }

    /** @test */
    public function admin_can_list_reviews(): void
    {
        // Arrange: Create some reviews
        Review::factory()->count(5)->create(['rating' => 3]);
        Review::factory()->count(3)->create(['rating' => 5]);

        // Act: Call the endpoint
        $response = $this->getJson('/api/admin/reviews');

        // Assert
        $response
            ->assertStatus(200)
            ->assertJsonStructure([
                'data' => [
                    '*' => ['id', 'rating', 'comment', 'name', 'email', 'created_at', 'updated_at']
                ],
                'links', // Pagination links
                'meta' // Pagination meta
            ])
            ->assertJsonCount(8, 'data'); // Check total number before pagination kicks in by default
    }

    // TODO: Add tests for listing bug reports and hardware surveys
    // TODO: Add tests for pagination (checking links, meta, different page results)
    // TODO: Add tests for filtering (e.g., /reviews?rating=5 returns only 5-star reviews)
    // TODO: Add tests for non-admin access (should fail, 403)
    // TODO: Add tests for unauthenticated access (should fail, 401)
}
