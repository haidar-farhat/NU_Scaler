<?php

namespace Tests\Feature\Api;

use App\Models\BugReport;
use App\Models\User;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Illuminate\Foundation\Testing\WithFaker;
use Laravel\Sanctum\Sanctum;
use Tests\TestCase;

class BugReportApiTest extends TestCase
{
    use RefreshDatabase, WithFaker;

    /**
     * Test that a bug report can be submitted by a guest.
     *
     * @return void
     */
    public function test_guest_can_submit_bug_report(): void
    {
        $bugReportData = [
            'description' => 'The application crashes when processing large files',
            'category' => 'crash',
            'severity' => 'high',
            'steps_to_reproduce' => 'Open a large file and click process',
            'system_info' => [
                'os' => 'Windows 11',
                'browser' => 'Chrome 100',
                'device' => 'Desktop',
                'app_version' => '1.2.0',
            ],
        ];

        $response = $this->postJson('/api/v1/feedback/bug-reports', $bugReportData);

        $response->assertStatus(201)
            ->assertJsonStructure([
                'message',
                'data' => [
                    'id',
                    'description',
                    'category',
                    'severity',
                    'steps_to_reproduce',
                    'system_info',
                    'created_at',
                    'updated_at',
                ],
            ]);

        $this->assertDatabaseHas('bug_reports', [
            'description' => 'The application crashes when processing large files',
            'category' => 'crash',
            'severity' => 'high',
        ]);
    }

    /**
     * Test that a bug report can be submitted by an authenticated user.
     *
     * @return void
     */
    public function test_authenticated_user_can_submit_bug_report(): void
    {
        $user = User::factory()->create();
        Sanctum::actingAs($user);

        $bugReportData = [
            'description' => 'Performance issues with large batches',
            'category' => 'performance',
            'severity' => 'medium',
            'steps_to_reproduce' => 'Process multiple images in a batch',
            'system_info' => [
                'os' => 'Windows 10',
                'browser' => 'Firefox 98',
                'device' => 'Desktop',
                'app_version' => '1.2.0',
            ],
        ];

        $response = $this->postJson('/api/v1/feedback/bug-reports', $bugReportData);

        $response->assertStatus(201)
            ->assertJsonStructure([
                'message',
                'data' => [
                    'id',
                    'description',
                    'category',
                    'severity',
                    'user_uuid',
                    'created_at',
                    'updated_at',
                ],
            ]);

        $this->assertDatabaseHas('bug_reports', [
            'description' => 'Performance issues with large batches',
            'category' => 'performance',
            'severity' => 'medium',
            'user_uuid' => $user->uuid,
        ]);
    }

    /**
     * Test that bug report submission requires valid data.
     *
     * @return void
     */
    public function test_bug_report_validation(): void
    {
        $response = $this->postJson('/api/v1/feedback/bug-reports', [
            'description' => '', // Empty description
            'category' => 'invalid-category', // Invalid category
            'severity' => '', // Missing severity
        ]);

        $response->assertStatus(422)
            ->assertJsonValidationErrors(['description', 'category', 'severity', 'system_info']);
    }

    /**
     * Test that admins can view all bug reports.
     *
     * @return void
     */
    public function test_admin_can_view_all_bug_reports(): void
    {
        // Create an admin user
        $admin = User::factory()->create(['is_admin' => true]);
        Sanctum::actingAs($admin);

        // Create some bug reports
        BugReport::factory()->count(5)->create();

        $response = $this->getJson('/api/admin/bug-reports');

        $response->assertStatus(200)
            ->assertJsonStructure([
                'data',
                'current_page',
                'total',
                'per_page',
            ]);
    }

    /**
     * Test that non-admins cannot access admin bug reports.
     *
     * @return void
     */
    public function test_non_admin_cannot_access_admin_bug_reports(): void
    {
        // Create a regular user
        $user = User::factory()->create(['is_admin' => false]);
        Sanctum::actingAs($user);

        // Create some bug reports
        BugReport::factory()->count(5)->create();

        $response = $this->getJson('/api/admin/bug-reports');

        $response->assertStatus(403);
    }
}
