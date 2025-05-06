<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;
use Illuminate\Support\Facades\Hash;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Str;

return new class extends Migration
{
    /**
     * Run the migrations.
     */
    public function up(): void
    {
        // Create default admin users if they don't exist
        $this->createDefaultAdminUsers();

        // Create mock data for testing admin dashboard
        $this->createMockData();
    }

    /**
     * Create default admin users
     */
    private function createDefaultAdminUsers(): void
    {
        // Check if all required columns exist in the users table
        $hasUuidColumn = Schema::hasColumn('users', 'uuid');
        $hasAdminColumn = Schema::hasColumn('users', 'is_admin');
        $hasActiveColumn = Schema::hasColumn('users', 'is_active');

        $defaultAdmins = [
            [
                'name' => 'Admin',
                'email' => 'admin@nuscaler.com',
                'password' => Hash::make('password'),
                'email_verified_at' => now(),
            ],
            [
                'name' => 'Haydar',
                'email' => 'haydar@nuscaler.com',
                'password' => Hash::make('password'),
                'email_verified_at' => now(),
            ]
        ];

        foreach ($defaultAdmins as $admin) {
            // Add optional columns if they exist
            if ($hasUuidColumn) {
                $admin['uuid'] = Str::uuid()->toString();
            }

            if ($hasAdminColumn) {
                $admin['is_admin'] = true;
            }

            if ($hasActiveColumn) {
                $admin['is_active'] = true;
            }

            if (!DB::table('users')->where('email', $admin['email'])->exists()) {
                DB::table('users')->insert(array_merge($admin, [
                    'created_at' => now(),
                    'updated_at' => now(),
                ]));
            }
        }
    }

    /**
     * Create mock data for testing
     */
    private function createMockData(): void
    {
        // Check if we have enough users to create mock data
        if (DB::table('users')->count() < 2) {
            return;
        }

        // Create regular users if none exist
        if (DB::table('users')->where('is_admin', false)->count() < 5 && Schema::hasColumn('users', 'is_admin')) {
            $this->createMockUsers();
        }

        // Only create related data if the tables exist
        if (Schema::hasTable('reviews')) {
            if (DB::table('reviews')->count() < 10) {
                $this->createMockReviews();
            }
        }

        if (Schema::hasTable('bug_reports')) {
            if (DB::table('bug_reports')->count() < 10) {
                $this->createMockBugReports();
            }
        }

        if (Schema::hasTable('hardware_surveys')) {
            if (DB::table('hardware_surveys')->count() < 10) {
                $this->createMockHardwareSurveys();
            }
        }
    }

    /**
     * Create mock users
     */
    private function createMockUsers(): void
    {
        // Check if all required columns exist
        $hasUuidColumn = Schema::hasColumn('users', 'uuid');
        $hasAdminColumn = Schema::hasColumn('users', 'is_admin');
        $hasActiveColumn = Schema::hasColumn('users', 'is_active');

        $users = [
            [
                'name' => 'John Doe',
                'email' => 'john@example.com',
                'password' => Hash::make('password'),
                'email_verified_at' => now(),
            ],
            [
                'name' => 'Jane Smith',
                'email' => 'jane@example.com',
                'password' => Hash::make('password'),
                'email_verified_at' => now(),
            ],
            [
                'name' => 'Alice Johnson',
                'email' => 'alice@example.com',
                'password' => Hash::make('password'),
                'email_verified_at' => now(),
            ],
            [
                'name' => 'Bob Williams',
                'email' => 'bob@example.com',
                'password' => Hash::make('password'),
                'email_verified_at' => now(),
            ],
            [
                'name' => 'Carol Brown',
                'email' => 'carol@example.com',
                'password' => Hash::make('password'),
                'email_verified_at' => now(),
            ],
        ];

        foreach ($users as $user) {
            // Add optional columns if they exist
            if ($hasUuidColumn) {
                $user['uuid'] = Str::uuid()->toString();
            }

            if ($hasAdminColumn) {
                $user['is_admin'] = false;
            }

            if ($hasActiveColumn) {
                $user['is_active'] = true;
            }

            if (!DB::table('users')->where('email', $user['email'])->exists()) {
                DB::table('users')->insert(array_merge($user, [
                    'created_at' => now(),
                    'updated_at' => now(),
                ]));
            }
        }
    }

    /**
     * Create mock reviews
     */
    private function createMockReviews(): void
    {
        // Only proceed if we have the user_id column in reviews
        if (!Schema::hasColumn('reviews', 'user_id')) {
            return;
        }

        $users = DB::table('users')->where('is_admin', false)->pluck('id')->toArray();

        // If no regular users exist, get any users
        if (empty($users)) {
            $users = DB::table('users')->pluck('id')->toArray();
        }

        if (empty($users)) {
            return;
        }

        $reviews = [
            [
                'rating' => 5,
                'comment' => 'Great software, helped me optimize my PC significantly!',
            ],
            [
                'rating' => 4,
                'comment' => 'Very useful tool. Would be perfect with a few more features.',
            ],
            [
                'rating' => 5,
                'comment' => 'Excellent performance improvements after using this.',
            ],
            [
                'rating' => 3,
                'comment' => 'Good but could use a more intuitive interface.',
            ],
            [
                'rating' => 5,
                'comment' => 'My games run much smoother now. Thank you!',
            ],
            [
                'rating' => 4,
                'comment' => 'Solid tool for PC optimization.',
            ],
            [
                'rating' => 5,
                'comment' => 'Best optimization software I\'ve used so far.',
            ],
            [
                'rating' => 4,
                'comment' => 'Very helpful for my old computer.',
            ],
            [
                'rating' => 3,
                'comment' => 'Decent tool but had some compatibility issues.',
            ],
            [
                'rating' => 5,
                'comment' => 'Incredible performance boost on my gaming PC!',
            ],
        ];

        foreach ($reviews as $review) {
            $userId = $users[array_rand($users)];

            DB::table('reviews')->insert(array_merge($review, [
                'user_id' => $userId,
                'created_at' => now()->subDays(rand(1, 30)),
                'updated_at' => now()->subDays(rand(0, 5)),
            ]));
        }
    }

    /**
     * Create mock bug reports
     */
    private function createMockBugReports(): void
    {
        // Only proceed if we have the user_id column in bug_reports
        if (!Schema::hasColumn('bug_reports', 'user_id')) {
            return;
        }

        $users = DB::table('users')->where('is_admin', false)->pluck('id')->toArray();

        // If no regular users exist, get any users
        if (empty($users)) {
            $users = DB::table('users')->pluck('id')->toArray();
        }

        if (empty($users)) {
            return;
        }

        $statuses = ['pending', 'in_progress', 'resolved', 'closed'];
        $severities = ['low', 'medium', 'high', 'critical'];
        $categories = ['UI', 'Performance', 'Compatibility', 'Feature', 'General'];

        $bugReports = [
            [
                'description' => 'The app crashes when I try to scan my system with more than 3 drives.',
                'severity' => $severities[array_rand($severities)],
                'category' => $categories[array_rand($categories)],
                'steps_to_reproduce' => 'Connect at least 4 drives, start a system scan, wait until it crashes.',
                'system_info' => json_encode(['os' => 'Windows 11', 'ram' => '32GB', 'cpu' => 'Intel i7']),
            ],
            [
                'description' => 'Memory optimization feature doesn\'t seem to be working on Windows 11.',
                'severity' => $severities[array_rand($severities)],
                'category' => $categories[array_rand($categories)],
                'steps_to_reproduce' => 'Run memory optimization on Windows 11.',
                'system_info' => json_encode(['os' => 'Windows 11', 'ram' => '16GB', 'cpu' => 'AMD Ryzen 5']),
            ],
            [
                'description' => 'The interface becomes unresponsive during disk cleanup.',
                'severity' => $severities[array_rand($severities)],
                'category' => $categories[array_rand($categories)],
                'steps_to_reproduce' => 'Start disk cleanup on a drive with more than 500GB.',
                'system_info' => json_encode(['os' => 'Windows 10', 'ram' => '8GB', 'cpu' => 'Intel i5']),
            ],
            [
                'description' => 'Getting an unhandled exception when trying to optimize network settings.',
                'severity' => $severities[array_rand($severities)],
                'category' => $categories[array_rand($categories)],
                'steps_to_reproduce' => 'Go to network settings, click optimize.',
                'system_info' => json_encode(['os' => 'Windows 11', 'ram' => '32GB', 'cpu' => 'AMD Ryzen 7']),
            ],
            [
                'description' => 'Custom optimization profiles don\'t save properly.',
                'severity' => $severities[array_rand($severities)],
                'category' => $categories[array_rand($categories)],
                'steps_to_reproduce' => 'Create a custom profile, save it, restart app, check if it still exists.',
                'system_info' => json_encode(['os' => 'Windows 10', 'ram' => '16GB', 'cpu' => 'Intel i9']),
            ],
            [
                'description' => 'Sometimes the application hangs indefinitely when starting up.',
                'severity' => $severities[array_rand($severities)],
                'category' => $categories[array_rand($categories)],
                'steps_to_reproduce' => 'Start the application multiple times.',
                'system_info' => json_encode(['os' => 'Windows 11', 'ram' => '64GB', 'cpu' => 'AMD Ryzen 9']),
            ],
            [
                'description' => 'The CPU optimization only detects half of my CPU cores.',
                'severity' => $severities[array_rand($severities)],
                'category' => $categories[array_rand($categories)],
                'steps_to_reproduce' => 'Run CPU optimization on a 12-core processor.',
                'system_info' => json_encode(['os' => 'Windows 10', 'ram' => '32GB', 'cpu' => 'Intel i7']),
            ],
            [
                'description' => 'When I try to export optimization results, nothing happens.',
                'severity' => $severities[array_rand($severities)],
                'category' => $categories[array_rand($categories)],
                'steps_to_reproduce' => 'Run optimization, click export results button.',
                'system_info' => json_encode(['os' => 'Windows 11', 'ram' => '16GB', 'cpu' => 'AMD Ryzen 5']),
            ],
            [
                'description' => 'The app consistently crashes with my RTX 4090 graphics card.',
                'severity' => $severities[array_rand($severities)],
                'category' => $categories[array_rand($categories)],
                'steps_to_reproduce' => 'Just start the app with RTX 4090 installed.',
                'system_info' => json_encode(['os' => 'Windows 11', 'ram' => '64GB', 'cpu' => 'Intel i9', 'gpu' => 'RTX 4090']),
            ],
            [
                'description' => 'All my settings were reset after the latest update.',
                'severity' => $severities[array_rand($severities)],
                'category' => $categories[array_rand($categories)],
                'steps_to_reproduce' => 'Update to latest version, check settings.',
                'system_info' => json_encode(['os' => 'Windows 10', 'ram' => '16GB', 'cpu' => 'Intel i5']),
            ],
        ];

        foreach ($bugReports as $bugReport) {
            $userId = $users[array_rand($users)];

            DB::table('bug_reports')->insert(array_merge($bugReport, [
                'user_id' => $userId,
                'created_at' => now()->subDays(rand(1, 30)),
                'updated_at' => now()->subDays(rand(0, 5)),
            ]));
        }
    }

    /**
     * Create mock hardware surveys
     */
    private function createMockHardwareSurveys(): void
    {
        // Only proceed if we have the user_id column in hardware_surveys
        if (!Schema::hasColumn('hardware_surveys', 'user_id')) {
            return;
        }

        $users = DB::table('users')->where('is_admin', false)->pluck('id')->toArray();

        // If no regular users exist, get any users
        if (empty($users)) {
            $users = DB::table('users')->pluck('id')->toArray();
        }

        if (empty($users)) {
            return;
        }

        $cpus = ['Intel Core i9-13900K', 'AMD Ryzen 9 7950X', 'Intel Core i7-12700K', 'AMD Ryzen 7 5800X', 'Intel Core i5-13600K'];
        $gpus = ['NVIDIA RTX 4090', 'AMD Radeon RX 7900 XTX', 'NVIDIA RTX 3080', 'AMD Radeon RX 6800 XT', 'NVIDIA RTX 4070'];
        $rams = ['16GB DDR4-3200', '32GB DDR4-3600', '64GB DDR5-5600', '8GB DDR4-2666', '32GB DDR5-6000'];
        $storage = ['1TB NVMe SSD', '2TB SATA SSD + 4TB HDD', '500GB NVMe SSD + 2TB HDD', '4TB NVMe SSD', '1TB SATA SSD'];

        for ($i = 0; $i < 10; $i++) {
            $userId = $users[array_rand($users)];

            DB::table('hardware_surveys')->insert([
                'user_id' => $userId,
                'cpu' => $cpus[array_rand($cpus)],
                'gpu' => $gpus[array_rand($gpus)],
                'ram' => $rams[array_rand($rams)],
                'storage' => $storage[array_rand($storage)],
                'os_version' => 'Windows ' . rand(10, 11) . (rand(0, 1) ? ' Pro' : ' Home'),
                'created_at' => now()->subDays(rand(1, 30)),
                'updated_at' => now(),
            ]);
        }
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        // We don't remove the data in rollback
    }
};
