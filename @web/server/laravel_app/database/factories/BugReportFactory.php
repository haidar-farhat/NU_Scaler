<?php

namespace Database\Factories;

use App\Models\BugReport;
use App\Models\User;
use Illuminate\Database\Eloquent\Factories\Factory;

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
        $categories = ['ui', 'performance', 'feature', 'crash', 'other'];
        $severities = ['low', 'medium', 'high', 'critical'];

        $descriptions = [
            'Application crashes when trying to upscale images larger than 8K resolution.',
            'UI freezes momentarily when switching between tabs.',
            'Memory leak when processing multiple batches in sequence.',
            'Cannot save presets with special characters in the name.',
            'Dark mode doesn\'t apply to all elements in the settings panel.',
            'Progress indicator doesn\'t match actual progress during batch processing.',
            'Some EXIF data is lost after upscaling.',
            'Export dialog doesn\'t remember last used location.',
            'Application becomes unresponsive when GPU is at 100% usage.',
            'Keyboard shortcuts stop working after using fullscreen preview.',
        ];

        $systems = $this->faker->randomElement([
            'Windows 10',
            'Windows 11',
            'macOS Monterey',
            'macOS Ventura',
            'Ubuntu 22.04',
            'Fedora 36',
        ]);

        return [
            'description' => $this->faker->randomElement($descriptions),
            'category' => $this->faker->randomElement($categories),
            'severity' => $this->faker->randomElement($severities),
            'steps_to_reproduce' => $this->faker->optional(0.8)->paragraph(),
            'system_info' => [
                'os' => $systems,
                'browser' => $this->faker->userAgent(),
                'device' => 'Desktop',
                'app_version' => '1.' . $this->faker->numberBetween(0, 5) . '.' . $this->faker->numberBetween(0, 9),
            ],
            'user_uuid' => User::factory()->create()->uuid,
            'created_at' => $this->faker->dateTimeBetween('-6 months', 'now'),
            'updated_at' => function (array $attributes) {
                return $attributes['created_at'];
            },
        ];
    }

    /**
     * Indicate that the bug report has a specific severity.
     *
     * @param string $severity
     * @return \Illuminate\Database\Eloquent\Factories\Factory
     */
    public function severity(string $severity)
    {
        return $this->state(fn (array $attributes) => [
            'severity' => $severity,
        ]);
    }

    /**
     * Indicate that the bug report has a specific category.
     *
     * @param string $category
     * @return \Illuminate\Database\Eloquent\Factories\Factory
     */
    public function category(string $category)
    {
        return $this->state(fn (array $attributes) => [
            'category' => $category,
        ]);
    }

    /**
     * Indicate that the bug report is critical.
     *
     * @return \Illuminate\Database\Eloquent\Factories\Factory
     */
    public function critical()
    {
        return $this->state(fn (array $attributes) => [
            'severity' => 'critical',
        ]);
    }

    /**
     * Indicate that the bug report is anonymous.
     *
     * @return \Illuminate\Database\Eloquent\Factories\Factory
     */
    public function anonymous()
    {
        return $this->state(fn (array $attributes) => [
            'user_uuid' => null,
        ]);
    }
}
