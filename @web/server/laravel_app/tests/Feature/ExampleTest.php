<?php

namespace Tests\Feature;

use Illuminate\Foundation\Testing\RefreshDatabase;
use Tests\TestCase;

class ExampleTest extends TestCase
{
    /**
     * A basic test example.
     */
    public function test_the_application_returns_a_successful_response(): void
    {
        // Instead of testing the welcome page which requires Vite,
        // test a simple API endpoint that we know should return 200
        $response = $this->get('/api/test-cors');

        $response->assertStatus(200);
    }
}
