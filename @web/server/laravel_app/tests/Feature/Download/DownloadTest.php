<?php

namespace Tests\Feature\Download;

use Illuminate\Foundation\Testing\RefreshDatabase;
// use Illuminate\Foundation\Testing\WithFaker;
use Tests\TestCase;
use App\Models\User;
use Laravel\Sanctum\Sanctum;
use App\Models\DownloadLog;

class DownloadTest extends TestCase
{
    use RefreshDatabase;

    /** @test */
    public function authenticated_user_can_get_download_info_and_log_is_created(): void
    {
        // Arrange: Create and authenticate a regular user
        $user = User::factory()->create(['is_admin' => false]);
        Sanctum::actingAs($user);

        $this->assertEquals(0, DownloadLog::count()); // Ensure no logs exist initially

        // Act
        $response = $this->getJson('/api/v1/download');

        // Assert
        $response
            ->assertStatus(200)
            ->assertJsonStructure([
                'message',
                'installer_url', // Check for placeholder keys
                'version',
            ]);

        // Assert log was created
        $this->assertEquals(1, DownloadLog::count());
        $log = DownloadLog::first();
        $this->assertEquals($user->id, $log->user_id);
        $this->assertNotNull($log->ip_address);
    }

    /** @test */
    public function unauthenticated_user_cannot_get_download_info(): void
    {
        $response = $this->getJson('/api/v1/download');

        $response->assertStatus(401);
    }

    // TODO: Test rate limiting for downloads
}
