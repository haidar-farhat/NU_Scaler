<?php

namespace Tests\Feature\Api;

use App\Models\BugReport;
use App\Models\HardwareSurvey;
use App\Models\Review;
use App\Models\User;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Illuminate\Foundation\Testing\WithFaker;
use Laravel\Sanctum\Sanctum;
use Tests\TestCase;

class AdminMetricsApiTest extends TestCase
{
    use RefreshDatabase, WithFaker;

    /**
     * Test that an admin can view dashboard metrics.
     *
     * @return void
     */
    public function test_admin_can_view_dashboard_metrics(): void
    {
        // Create an admin user
        $admin = User::factory()->create(['is_admin' => true]);
        Sanctum::actingAs($admin);

        // Create some test data
        $userCount = 5;
        $reviewCount = 3;
        $bugReportCount = 2;
        $hardwareSurveyCount = 4;

        User::factory()->count($userCount)->create();
        Review::factory()->count($reviewCount)->create();
        BugReport::factory()->count($bugReportCount)->create();
        HardwareSurvey::factory()->count($hardwareSurveyCount)->create();

        $response = $this->getJson('/api/admin/metrics/dashboard');

        $response->assertStatus(200)
            ->assertJsonStructure([
                'data' => [
                    'users' => [
                        'total',
                        'new_today',
                    ],
                    'reviews' => [
                        'total',
                        'average_rating',
                        'new_today',
                    ],
                    'bug_reports' => [
                        'total',
                        'new_today',
                        'by_severity',
                    ],
                    'hardware_surveys' => [
                        'total',
                        'new_today',
                    ],
                ],
            ]);

        // Verify reviews, bug reports and hardware surveys counts
        // These are created in this test so counts should be reliable
        $responseData = $response->json('data');
        $this->assertEquals($reviewCount, $responseData['reviews']['total'], 'Review count mismatch');
        $this->assertEquals($bugReportCount, $responseData['bug_reports']['total'], 'Bug report count mismatch');
        $this->assertEquals($hardwareSurveyCount, $responseData['hardware_surveys']['total'], 'Hardware survey count mismatch');

        // For user count, just verify it's at least the count we created plus admin (some may exist from other tests)
        $this->assertGreaterThanOrEqual($userCount + 1, $responseData['users']['total'], 'User count too low');
    }

    /**
     * Test that an admin can view review metrics.
     *
     * @return void
     */
    public function test_admin_can_view_review_metrics(): void
    {
        // Create an admin user
        $admin = User::factory()->create(['is_admin' => true]);
        Sanctum::actingAs($admin);

        // Create some reviews with different ratings
        Review::factory()->count(2)->create(['rating' => 5]);
        Review::factory()->count(3)->create(['rating' => 4]);
        Review::factory()->count(1)->create(['rating' => 3]);

        $response = $this->getJson('/api/admin/metrics/reviews');

        $response->assertStatus(200)
            ->assertJsonStructure([
                'data' => [
                    'average_rating',
                    'total_reviews',
                    'ratings_distribution',
                    'sentiment',
                ],
            ]);

        // Verify correct counts and average
        $this->assertEquals(6, $response->json('data.total_reviews'));

        // Average should be (5*2 + 4*3 + 3*1)/6 = 4.17
        $this->assertEqualsWithDelta(4.17, $response->json('data.average_rating'), 0.01);

        // Verify distribution
        $this->assertArrayHasKey('5', $response->json('data.ratings_distribution'));
        $this->assertArrayHasKey('4', $response->json('data.ratings_distribution'));
        $this->assertArrayHasKey('3', $response->json('data.ratings_distribution'));
        $this->assertEquals(2, $response->json('data.ratings_distribution.5'));
        $this->assertEquals(3, $response->json('data.ratings_distribution.4'));
        $this->assertEquals(1, $response->json('data.ratings_distribution.3'));
    }

    /**
     * Test that a non-admin cannot access metrics.
     *
     * @return void
     */
    public function test_non_admin_cannot_access_metrics(): void
    {
        // Create a regular user
        $user = User::factory()->create(['is_admin' => false]);
        Sanctum::actingAs($user);

        $response = $this->getJson('/api/admin/metrics/dashboard');
        $response->assertStatus(403);

        $response = $this->getJson('/api/admin/metrics/reviews');
        $response->assertStatus(403);

        $response = $this->getJson('/api/admin/metrics/bug-reports');
        $response->assertStatus(403);

        $response = $this->getJson('/api/admin/metrics/hardware-surveys');
        $response->assertStatus(403);
    }

    /**
     * Test that metrics can be exported.
     *
     * @return void
     */
    public function test_metrics_export(): void
    {
        // Create an admin user
        $admin = User::factory()->create(['is_admin' => true]);
        Sanctum::actingAs($admin);

        // Create some test data
        Review::factory()->count(3)->create();
        BugReport::factory()->count(2)->create();
        HardwareSurvey::factory()->count(4)->create();

        $response = $this->getJson('/api/admin/metrics/export');

        $response->assertStatus(200)
            ->assertJsonStructure([
                'data' => [
                    'reviews',
                    'bug_reports',
                    'hardware_surveys',
                ],
                'generated_at',
                'version',
            ]);
    }
}
