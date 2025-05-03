<?php

namespace Database\Factories;

use Illuminate\Database\Eloquent\Factories\Factory;
use App\Models\HardwareSurvey;

/**
 * @extends \Illuminate\Database\Eloquent\Factories\Factory<\App\Models\HardwareSurvey>
 */
class HardwareSurveyFactory extends Factory
{
    /**
     * The name of the factory's corresponding model.
     *
     * @var string
     */
    protected $model = HardwareSurvey::class;

    /**
     * Define the model's default state.
     *
     * @return array<string, mixed>
     */
    public function definition(): array
    {
        return [
            'cpu' => $this->faker->randomElement(['Intel Core i7-13700K', 'AMD Ryzen 7 7800X3D', 'Intel Core i5-12600K', 'Apple M2 Pro']),
            'gpu' => $this->faker->randomElement(['NVIDIA GeForce RTX 4090', 'NVIDIA GeForce RTX 3080', 'AMD Radeon RX 7900 XTX', 'Intel Arc A770']),
            'ram_gb' => $this->faker->randomElement([16, 32, 64]),
            'os' => $this->faker->randomElement(['Windows 11', 'Windows 10', 'MacOS Sonoma', 'Ubuntu 22.04']),
            'resolution' => $this->faker->randomElement(['1920x1080', '2560x1440', '3840x2160']),
            'user_uuid' => null,
            'created_at' => now(),
            'updated_at' => now(),
        ];
    }
}
