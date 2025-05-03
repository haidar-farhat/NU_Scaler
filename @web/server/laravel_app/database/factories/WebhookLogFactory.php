<?php

namespace Database\Factories;

use App\Models\Webhook;
use App\Models\WebhookLog;
use Illuminate\Database\Eloquent\Factories\Factory;

/**
 * @extends \Illuminate\Database\Eloquent\Factories\Factory<\App\Models\WebhookLog>
 */
class WebhookLogFactory extends Factory
{
    /**
     * The name of the factory's corresponding model.
     *
     * @var string
     */
    protected $model = WebhookLog::class;

    /**
     * Define the model's default state.
     *
     * @return array<string, mixed>
     */
    public function definition(): array
    {
        $eventTypes = [
            'feedback.review.created',
            'feedback.bug.created',
            'feedback.hardware.created',
            'user.registered',
            'ping',
        ];

        $statusCodes = [200, 201, 204, 400, 401, 403, 404, 500, 502, 504];
        $isSuccess = $this->faker->boolean(80); // 80% chance of success

        return [
            'webhook_id' => Webhook::factory(),
            'event_type' => $this->faker->randomElement($eventTypes),
            'payload' => [
                'event' => $this->faker->randomElement($eventTypes),
                'id' => $this->faker->randomNumber(5),
                'timestamp' => $this->faker->dateTimeThisMonth()->format('Y-m-d H:i:s'),
                'data' => [
                    'key' => $this->faker->word(),
                    'value' => $this->faker->sentence(),
                ],
            ],
            'status_code' => $isSuccess
                ? $this->faker->randomElement([200, 201, 204])
                : $this->faker->randomElement([400, 401, 403, 404, 500, 502, 504]),
            'response' => $isSuccess
                ? json_encode(['status' => 'success', 'message' => 'Webhook received'])
                : json_encode(['status' => 'error', 'message' => 'Failed to process webhook']),
            'error' => $isSuccess ? null : $this->faker->sentence(),
            'success' => $isSuccess,
            'created_at' => $this->faker->dateTimeThisMonth(),
            'updated_at' => function (array $attributes) {
                return $attributes['created_at'];
            },
        ];
    }

    /**
     * Indicate that the webhook log was successful.
     *
     * @return \Illuminate\Database\Eloquent\Factories\Factory
     */
    public function successful()
    {
        return $this->state(fn (array $attributes) => [
            'status_code' => $this->faker->randomElement([200, 201, 204]),
            'response' => json_encode(['status' => 'success', 'message' => 'Webhook received']),
            'error' => null,
            'success' => true,
        ]);
    }

    /**
     * Indicate that the webhook log failed.
     *
     * @return \Illuminate\Database\Eloquent\Factories\Factory
     */
    public function failed()
    {
        return $this->state(fn (array $attributes) => [
            'status_code' => $this->faker->randomElement([400, 401, 403, 404, 500, 502, 504]),
            'response' => json_encode(['status' => 'error', 'message' => 'Failed to process webhook']),
            'error' => $this->faker->sentence(),
            'success' => false,
        ]);
    }

    /**
     * Indicate a specific event type.
     *
     * @param string $eventType
     * @return \Illuminate\Database\Eloquent\Factories\Factory
     */
    public function forEvent(string $eventType)
    {
        return $this->state(fn (array $attributes) => [
            'event_type' => $eventType,
            'payload' => [
                'event' => $eventType,
                'id' => $this->faker->randomNumber(5),
                'timestamp' => $this->faker->dateTimeThisMonth()->format('Y-m-d H:i:s'),
                'data' => [
                    'key' => $this->faker->word(),
                    'value' => $this->faker->sentence(),
                ],
            ],
        ]);
    }
}
