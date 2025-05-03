<?php

namespace Database\Factories;

use Illuminate\Database\Eloquent\Factories\Factory;
use App\Models\BugReport;

/**
 * @extends \Illuminate\Database\Eloquent\Factories\Factory<\App\Models\BugReport>
 */
class BugReportFactory extends Factory
{
    /**
     * The name of the factory's corresponding model.
     *
     * @var string
     */
    protected $model = BugReport::class;

    /**
     * Define the model's default state.
     *
     * @return array<string, mixed>
     */
    public function definition(): array
    {
        return [
            'description' => $this->faker->text(500),
            'severity' => $this->faker->randomElement(['low', 'medium', 'high', 'critical']),
            'log_path' => null,
            'system_info' => [
                'os' => $this->faker->randomElement(['Windows 11', 'Windows 10', 'MacOS Sonoma', 'Ubuntu 22.04']),
                'version' => 'App Version ' . $this->faker->semver(),
            ],
            'user_uuid' => null,
            'created_at' => now(),
            'updated_at' => now(),
        ];
    }
}
