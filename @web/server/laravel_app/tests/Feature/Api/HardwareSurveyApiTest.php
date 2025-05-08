<?php

namespace Tests\Feature\Api;

use App\Models\HardwareSurvey;
use App\Models\User;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Illuminate\Foundation\Testing\WithFaker;
use Laravel\Sanctum\Sanctum;
use Tests\TestCase;

class HardwareSurveyApiTest extends TestCase
{
    use RefreshDatabase, WithFaker;

    /**
     * Test that a hardware survey can be submitted by a guest.
     *
     * @return void
     */
    public function test_guest_can_submit_hardware_survey(): void
    {
        $surveyData = [
            'cpu_model' => 'Intel Core i7-12700K',
            'gpu_model' => 'NVIDIA RTX 3080',
            'ram_size' => 32,
            'os' => 'Windows 11',
            'resolution' => '3840x2160',
            'monitor_refresh_rate' => 144,
            'additional_info' => 'Running with dual monitors',
        ];

        $response = $this->postJson('/api/v1/feedback/hardware-surveys', $surveyData);

        $response->assertStatus(201)
            ->assertJsonStructure([
                'message',
                'data' => [
                    'id',
                    'cpu_model',
                    'gpu_model',
                    'ram_size',
                    'os',
                    'resolution',
                    'monitor_refresh_rate',
                    'additional_info',
                    'created_at',
                    'updated_at',
                ],
            ]);

        $this->assertDatabaseHas('hardware_surveys', [
            'cpu_model' => 'Intel Core i7-12700K',
            'gpu_model' => 'NVIDIA RTX 3080',
            'ram_size' => 32,
            'os' => 'Windows 11',
        ]);
    }

    /**
     * Test that a hardware survey can be submitted by an authenticated user.
     *
     * @return void
     */
    public function test_authenticated_user_can_submit_hardware_survey(): void
    {
        $user = User::factory()->create();
        Sanctum::actingAs($user);

        $surveyData = [
            'cpu_model' => 'AMD Ryzen 9 5900X',
            'gpu_model' => 'AMD Radeon RX 6800 XT',
            'ram_size' => 64,
            'os' => 'Windows 10',
            'resolution' => '3440x1440',
            'monitor_refresh_rate' => 165,
            'additional_info' => 'Using custom cooling solution',
        ];

        $response = $this->postJson('/api/v1/feedback/hardware-surveys', $surveyData);

        $response->assertStatus(201)
            ->assertJsonStructure([
                'message',
                'data' => [
                    'id',
                    'cpu_model',
                    'gpu_model',
                    'ram_size',
                    'os',
                    'resolution',
                    'monitor_refresh_rate',
                    'additional_info',
                    'user_uuid',
                    'created_at',
                    'updated_at',
                ],
            ]);

        $this->assertDatabaseHas('hardware_surveys', [
            'cpu_model' => 'AMD Ryzen 9 5900X',
            'gpu_model' => 'AMD Radeon RX 6800 XT',
            'ram_size' => 64,
            'user_uuid' => $user->uuid,
        ]);
    }

    /**
     * Test that hardware survey submission requires valid data.
     *
     * @return void
     */
    public function test_hardware_survey_validation(): void
    {
        $response = $this->postJson('/api/v1/feedback/hardware-surveys', [
            'cpu_model' => '', // Empty CPU model
            'gpu_model' => '', // Empty GPU model
            'ram_size' => 'not-a-number', // Invalid RAM size
            'os' => '', // Empty OS
        ]);

        $response->assertStatus(422)
            ->assertJsonValidationErrors(['cpu_model', 'gpu_model', 'ram_size', 'os', 'resolution']);
    }

    /**
     * Test that admins can view all hardware surveys.
     *
     * @return void
     */
    public function test_admin_can_view_all_hardware_surveys(): void
    {
        // Create an admin user
        $admin = User::factory()->create(['is_admin' => true]);
        Sanctum::actingAs($admin);

        // Create some hardware surveys
        HardwareSurvey::factory()->count(5)->create();

        $response = $this->getJson('/api/admin/hardware-surveys');

        $response->assertStatus(200)
            ->assertJsonStructure([
                'data',
                'current_page',
                'total',
                'per_page',
            ]);
    }

    /**
     * Test that non-admins cannot access admin hardware surveys.
     *
     * @return void
     */
    public function test_non_admin_cannot_access_admin_hardware_surveys(): void
    {
        // Create a regular user
        $user = User::factory()->create(['is_admin' => false]);
        Sanctum::actingAs($user);

        // Create some hardware surveys
        HardwareSurvey::factory()->count(5)->create();

        $response = $this->getJson('/api/admin/hardware-surveys');

        $response->assertStatus(403);
    }
}
