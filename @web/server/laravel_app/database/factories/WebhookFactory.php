<?php

namespace Database\Factories;

use App\Models\User;
use App\Models\Webhook;
use Illuminate\Database\Eloquent\Factories\Factory;

/**
 * @extends \Illuminate\Database\Eloquent\Factories\Factory<\App\Models\Webhook>
 */
class WebhookFactory extends Factory
{
    /**
     * The name of the factory's corresponding model.
     *
     * @var string
     */
    protected $model = Webhook::class;

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
        ];

        return [
            'name' => $this->faker->words(3, true) . ' Webhook',
            'url' => $this->faker->url(),
            'description' => $this->faker->sentence(),
            'is_active' => true,
            'events' => $this->faker->randomElements($eventTypes, $this->faker->numberBetween(1, count($eventTypes))),
            'secret' => Webhook::generateSecret(),
            'user_id' => User::factory(),
            'headers' => [
                'Content-Type' => 'application/json',
                'X-Custom-Header' => $this->faker->word(),
            ],
            'fails_count' => 0,
        ];
    }

    /**
     * Indicate that the webhook is inactive.
     *
     * @return \Illuminate\Database\Eloquent\Factories\Factory
     */
    public function inactive()
    {
        return $this->state(fn (array $attributes) => [
            'is_active' => false,
        ]);
    }

    /**
     * Indicate that the webhook has failed several times.
     *
     * @param int $count
     * @return \Illuminate\Database\Eloquent\Factories\Factory
     */
    public function withFailures(int $count = 5)
    {
        return $this->state(fn (array $attributes) => [
            'fails_count' => $count,
        ]);
    }

    /**
     * Indicate that the webhook is subscribed to all events.
     *
     * @return \Illuminate\Database\Eloquent\Factories\Factory
     */
    public function allEvents()
    {
        return $this->state(fn (array $attributes) => [
            'events' => ['*'],
        ]);
    }
}
