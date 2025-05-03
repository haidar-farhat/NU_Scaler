<?php

namespace Tests\Feature\Feedback;

use Illuminate\Foundation\Testing\RefreshDatabase;
use Illuminate\Foundation\Testing\WithFaker;
use Tests\TestCase;
use App\Models\Review;

class PublicFeedbackSubmissionTest extends TestCase
{
    use RefreshDatabase;

    /**
     * A basic feature test example.
     */
    public function test_example(): void
    {
        $response = $this->get('/');

        $response->assertStatus(200);
    }

    /** @test */
    public function user_can_submit_a_review_successfully(): void
    {
        $reviewData = [
            'rating' => 5,
            'comment' => 'Excellent work!',
            'name' => 'Test Submitter',
            'email' => 'submitter@example.com',
        ];

        $response = $this->postJson('/api/v1/feedback/reviews', $reviewData);

        $response
            ->assertStatus(201)
            ->assertJsonStructure([
                'message',
                'data' => ['id', 'rating', 'comment', 'name', 'email', 'created_at', 'updated_at'],
            ])
            ->assertJsonPath('data.rating', 5);

        $this->assertDatabaseHas('reviews', [
            'rating' => 5,
            'comment' => 'Excellent work!',
            'email' => 'submitter@example.com',
        ]);
    }

    // TODO: Add tests for bug report and hardware survey submissions
    // TODO: Add tests for validation failures (e.g., missing rating, invalid severity)
}
