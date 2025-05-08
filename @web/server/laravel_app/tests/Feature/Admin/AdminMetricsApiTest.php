<?php

namespace Tests\Feature\Admin;

use Illuminate\Foundation\Testing\RefreshDatabase;
// use Illuminate\Foundation\Testing\WithFaker; // Not needed if not using Faker directly
use Tests\TestCase;
use App\Models\User;
use App\Models\Review;
use Laravel\Sanctum\Sanctum;

class AdminMetricsApiTest extends TestCase
{
    use RefreshDatabase;

    protected User $adminUser;

    protected function setUp(): void
    {
        parent::setUp();
        $this->adminUser = User::factory()->create(['is_admin' => true]);
        Sanctum::actingAs($this->adminUser, ['*']);
    }

    /** @test */
    public function admin_can_get_reviews_distribution(): void
    {
        // Arrange: Create reviews with different ratings
        Review::factory()->count(5)->create(['rating' => 1]);
        Review::factory()->count(10)->create(['rating' => 3]);
        Review::factory()->count(2)->create(['rating' => 5]);

        // Act
        $response = $this->getJson('/api/admin/metrics/reviews-distribution');

        // Assert
        $response
            ->assertStatus(200)
            ->assertJsonCount(3) // Expecting 3 rating groups (1, 3, 5)
            ->assertJsonFragment(['rating' => 1, 'count' => 5])
            ->assertJsonFragment(['rating' => 3, 'count' => 10])
            ->assertJsonFragment(['rating' => 5, 'count' => 2]);
    }

    // TODO: Add test for bug report severity distribution
    // TODO: Add tests for non-admin/unauthenticated access
}
