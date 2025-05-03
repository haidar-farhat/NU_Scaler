<?php

namespace Tests\Feature\Auth;

use Illuminate\Foundation\Testing\RefreshDatabase;
// use Illuminate\Foundation\Testing\WithFaker; // Not needed
use Tests\TestCase;
use App\Models\User;
use Illuminate\Support\Facades\Hash;

class AdminLoginTest extends TestCase
{
    use RefreshDatabase;

    /** @test */
    public function admin_user_can_login_successfully(): void
    {
        // Create an admin user
        $admin = User::factory()->create([
            'email' => 'admin@example.com',
            'password' => Hash::make('password'),
            'is_admin' => true,
        ]);

        $credentials = [
            'email' => 'admin@example.com',
            'password' => 'password',
        ];

        $response = $this->postJson(route('api.admin.login'), $credentials);

        $response
            ->assertStatus(200)
            ->assertJsonStructure([
                'message',
                'token_type',
                'access_token',
                'user' => ['id', 'name', 'email'],
            ])
            ->assertJsonPath('user.email', $admin->email);
    }

    // TODO: Add tests for non-admin login attempt (should fail, 403)
    // TODO: Add tests for wrong password (should fail, 422)
    // TODO: Add tests for non-existent user (should fail, 422)
}
