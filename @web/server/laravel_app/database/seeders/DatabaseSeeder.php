<?php

namespace Database\Seeders;

use App\Models\User;
use App\Models\Review;
use App\Models\BugReport;
use App\Models\HardwareSurvey;
// use Illuminate\Database\Console\Seeds\WithoutModelEvents;
use Illuminate\Database\Seeder;
use Illuminate\Support\Facades\DB;

class DatabaseSeeder extends Seeder
{
    /**
     * Seed the application's database.
     */
    public function run(): void
    {
        // User::factory(10)->create();

        // Use firstOrCreate to avoid duplicate entry for test user
        User::firstOrCreate(
            ['email' => 'test@example.com'],
            [
                'name' => 'Test User',
                'password' => bcrypt('testpassword'),
                'is_admin' => false,
            ]
        );

        // Seed an admin user if not present
        $admin = User::firstOrCreate(
            ['email' => 'admin@example.com'],
            [
                'name' => 'Admin User',
                'password' => bcrypt('adminpassword'),
                'is_admin' => true,
            ]
        );

        // Create test admin user matching someone's name for convenience
        User::firstOrCreate(
            ['email' => 'haydar@example.com'],
            [
                'name' => 'Haydar',
                'password' => bcrypt('password'),
                'is_admin' => true,
            ]
        );

        // Only seed demo data if tables are empty
        if (Review::count() === 0) {
            // Seed some demo reviews
            $ratings = [3, 4, 5, 5, 4];
            $comments = [
                'Great app, works as expected!',
                'The upscaling is impressive, but could use more options.',
                'I love this app, it saved me hours of manual work!',
                'Best upscaler I\'ve used so far. Worth the price.',
                'Very good results with my old photos.'
            ];

            foreach ($ratings as $index => $rating) {
                Review::create([
                    'user_id' => 1,
                    'rating' => $rating,
                    'comment' => $comments[$index],
                    'created_at' => now()->subDays(rand(1, 30))
                ]);
            }
        }

        if (BugReport::count() === 0) {
            // Seed some demo bug reports
            $severities = ['low', 'medium', 'high', 'critical', 'medium'];
            $categories = ['UI', 'Performance', 'Crash', 'Feature', 'Output'];
            $descriptions = [
                'Button alignment issue on the settings page.',
                'App runs slowly when processing multiple images.',
                'Application crashes when using more than 16GB RAM.',
                'Cannot save preferences after changing theme.',
                'Some images have artifacts after upscaling.'
            ];

            foreach ($severities as $index => $severity) {
                BugReport::create([
                    'user_id' => 1,
                    'severity' => $severity,
                    'category' => $categories[$index],
                    'description' => $descriptions[$index],
                    'created_at' => now()->subDays(rand(1, 30))
                ]);
            }
        }

        if (HardwareSurvey::count() === 0) {
            // Seed some demo hardware surveys
            $operatingSystems = ['Windows 11', 'macOS Monterey', 'Ubuntu 22.04', 'Windows 10', 'macOS Ventura'];
            $cpuModels = ['Intel i9-12900K', 'AMD Ryzen 9 5950X', 'Intel i7-11700K', 'Apple M1 Pro', 'AMD Ryzen 7 5800X'];
            $gpuModels = ['NVIDIA RTX 3080', 'AMD Radeon RX 6800 XT', 'NVIDIA RTX 3070', 'Apple M1 GPU', 'NVIDIA RTX 3060 Ti'];
            $ramSizes = [32, 16, 64, 16, 32];

            foreach ($operatingSystems as $index => $os) {
                HardwareSurvey::create([
                    'user_id' => 1,
                    'os' => $os,
                    'cpu_model' => $cpuModels[$index],
                    'gpu_model' => $gpuModels[$index],
                    'ram_size' => $ramSizes[$index],
                    'additional_info' => 'Survey submitted via app',
                    'created_at' => now()->subDays(rand(1, 30))
                ]);
            }
        }

        // Add user growth data for chart (created_at dates for users)
        // First, check if we only have our seeded users
        if (User::count() <= 3) {
            // Create 30 sample users with different creation dates
            for ($i = 1; $i <= 30; $i++) {
                User::create([
                    'name' => "Demo User $i",
                    'email' => "demo$i@example.com",
                    'password' => bcrypt('password'),
                    'is_admin' => false,
                    'created_at' => now()->subDays(rand(1, 90))
                ]);
            }
        }
    }
}
