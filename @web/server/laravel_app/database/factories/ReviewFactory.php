<?php

namespace Database\Factories;

use App\Models\Review;
use App\Models\User;
use Illuminate\Database\Eloquent\Factories\Factory;

/**
 * @extends \Illuminate\Database\Eloquent\Factories\Factory<\App\Models\Review>
 */
class ReviewFactory extends Factory
{
    /**
     * The name of the factory's corresponding model.
     *
     * @var string
     */
    protected $model = Review::class;

    /**
     * Define the model's default state.
     *
     * @return array<string, mixed>
     */
    public function definition(): array
    {
        $comments = [
            'Great application, really improved my workflow!',
            'Solid performance, but could use some UI improvements.',
            'The best upscaling tool I\'ve used so far.',
            'Works well on my system. Very impressed with the results.',
            'Good tool overall, but had some issues with large images.',
            'Love the new features in this version!',
            'Fantastic results, but a bit slow on my PC.',
            'The UI is much improved from previous versions.',
            'Simple to use and great results.',
            'Does exactly what it promises. Highly recommended.',
        ];

        return [
            'rating' => $this->faker->numberBetween(1, 5),
            'comment' => $this->faker->randomElement($comments),
            'name' => $this->faker->optional(0.7)->name(),
            'email' => $this->faker->optional(0.5)->safeEmail(),
            'user_uuid' => User::factory()->create()->uuid,
            'created_at' => $this->faker->dateTimeBetween('-6 months', 'now'),
            'updated_at' => function (array $attributes) {
                return $attributes['created_at'];
            },
        ];
    }

    /**
     * Configure the model factory.
     *
     * @return $this
     */
    public function configure()
    {
        return $this->afterMaking(function (Review $review) {
            // Any post-creation modifications if needed
        })->afterCreating(function (Review $review) {
            // Any post-creation actions if needed
        });
    }

    /**
     * Indicate that the review is by an anonymous user.
     *
     * @return \Illuminate\Database\Eloquent\Factories\Factory
     */
    public function anonymous()
    {
        return $this->state(fn (array $attributes) => [
            'name' => null,
            'email' => null,
            'user_uuid' => null,
        ]);
    }

    /**
     * Indicate that the review has a specific rating.
     *
     * @param int $rating
     * @return \Illuminate\Database\Eloquent\Factories\Factory
     */
    public function rating(int $rating)
    {
        return $this->state(fn (array $attributes) => [
            'rating' => max(1, min(5, $rating)),
        ]);
    }
}
