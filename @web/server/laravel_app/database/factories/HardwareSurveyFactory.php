<?php

namespace Database\Factories;

use App\Models\HardwareSurvey;
use App\Models\User;
use Illuminate\Database\Eloquent\Factories\Factory;

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
        $gpuBrands = ['NVIDIA', 'AMD', 'Intel'];
        $gpuModels = [
            'NVIDIA' => ['RTX 4090', 'RTX 4080', 'RTX 3090', 'RTX 3080', 'RTX 3070', 'RTX 3060', 'GTX 1080 Ti', 'GTX 1660 Super'],
            'AMD' => ['RX 7900 XTX', 'RX 7900 XT', 'RX 6900 XT', 'RX 6800 XT', 'RX 6700 XT', 'RX 5700 XT'],
            'Intel' => ['Arc A770', 'Arc A750', 'Arc A380'],
        ];

        $cpuBrands = ['Intel', 'AMD'];
        $cpuModels = [
            'Intel' => ['Core i9-13900K', 'Core i7-13700K', 'Core i5-13600K', 'Core i9-12900K', 'Core i7-12700K', 'Core i5-12600K'],
            'AMD' => ['Ryzen 9 7950X', 'Ryzen 7 7700X', 'Ryzen 5 7600X', 'Ryzen 9 5950X', 'Ryzen 7 5800X', 'Ryzen 5 5600X'],
        ];

        $operatingSystems = [
            'Windows 11',
            'Windows 10',
            'macOS Sonoma',
            'macOS Ventura',
            'Ubuntu 22.04',
            'Fedora 38',
        ];

        $resolutions = [
            '1920x1080',
            '2560x1440',
            '3440x1440',
            '3840x2160',
            '5120x1440',
            '7680x4320',
        ];

        $selectedGpuBrand = $this->faker->randomElement($gpuBrands);
        $selectedCpuBrand = $this->faker->randomElement($cpuBrands);

        return [
            'cpu_model' => $selectedCpuBrand . ' ' . $this->faker->randomElement($cpuModels[$selectedCpuBrand]),
            'gpu_model' => $selectedGpuBrand . ' ' . $this->faker->randomElement($gpuModels[$selectedGpuBrand]),
            'ram_size' => $this->faker->randomElement([8, 16, 32, 64, 128]),
            'os' => $this->faker->randomElement($operatingSystems),
            'resolution' => $this->faker->randomElement($resolutions),
            'monitor_refresh_rate' => $this->faker->randomElement([60, 75, 120, 144, 165, 240, 360]),
            'additional_info' => $this->faker->optional(0.3)->paragraph(),
            'user_uuid' => User::factory()->create()->uuid,
            'created_at' => $this->faker->dateTimeBetween('-6 months', 'now'),
            'updated_at' => function (array $attributes) {
                return $attributes['created_at'];
            },
        ];
    }

    /**
     * Indicate that the hardware survey has a specific GPU brand.
     *
     * @param string $brand
     * @return \Illuminate\Database\Eloquent\Factories\Factory
     */
    public function gpuBrand(string $brand)
    {
        $brands = [
            'nvidia' => 'NVIDIA',
            'amd' => 'AMD',
            'intel' => 'Intel',
        ];

        $normalizedBrand = strtolower($brand);
        $brandName = $brands[$normalizedBrand] ?? 'NVIDIA';

        $models = [
            'NVIDIA' => ['RTX 4090', 'RTX 4080', 'RTX 3090', 'RTX 3080', 'RTX 3070', 'RTX 3060', 'GTX 1080 Ti', 'GTX 1660 Super'],
            'AMD' => ['RX 7900 XTX', 'RX 7900 XT', 'RX 6900 XT', 'RX 6800 XT', 'RX 6700 XT', 'RX 5700 XT'],
            'Intel' => ['Arc A770', 'Arc A750', 'Arc A380'],
        ];

        return $this->state(fn (array $attributes) => [
            'gpu_model' => $brandName . ' ' . $this->faker->randomElement($models[$brandName]),
        ]);
    }

    /**
     * Indicate that the hardware survey has a specific RAM size.
     *
     * @param int $ramSize
     * @return \Illuminate\Database\Eloquent\Factories\Factory
     */
    public function ramSize(int $ramSize)
    {
        return $this->state(fn (array $attributes) => [
            'ram_size' => $ramSize,
        ]);
    }

    /**
     * Indicate that the hardware survey has a high-end configuration.
     *
     * @return \Illuminate\Database\Eloquent\Factories\Factory
     */
    public function highEnd()
    {
        return $this->state(fn (array $attributes) => [
            'gpu_model' => $this->faker->randomElement(['NVIDIA RTX 4090', 'NVIDIA RTX 4080', 'AMD RX 7900 XTX']),
            'cpu_model' => $this->faker->randomElement(['Intel Core i9-13900K', 'AMD Ryzen 9 7950X']),
            'ram_size' => $this->faker->randomElement([32, 64, 128]),
            'resolution' => $this->faker->randomElement(['3840x2160', '5120x1440', '7680x4320']),
        ]);
    }

    /**
     * Indicate that the hardware survey is anonymous.
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
