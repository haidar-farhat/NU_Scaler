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
        $defaultAdmins = [
            [
                'name' => 'Admin',
                'email' => 'admin@nuscaler.com',
                'password' => Hash::make('password'),
                'uuid' => Str::uuid()->toString(),
                'is_admin' => true,
                'is_active' => true,
                'email_verified_at' => now(),
            ],
            [
                'name' => 'Haydar',
                'email' => 'haydar@nuscaler.com',
                'password' => Hash::make('password'),
                'uuid' => Str::uuid()->toString(),
                'is_admin' => true,
                'is_active' => true,
                'email_verified_at' => now(),
            ]
        ];

        foreach ($defaultAdmins as $admin) {
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
        // Create regular users if none exist
        if (DB::table('users')->where('is_admin', false)->count() < 5) {
            $this->createMockUsers();
        }

        // Create reviews if none exist
        if (DB::table('reviews')->count() < 10) {
            $this->createMockReviews();
        }

        // Create bug reports if none exist
        if (DB::table('bug_reports')->count() < 10) {
            $this->createMockBugReports();
        }

        // Create hardware surveys if none exist
        if (DB::table('hardware_surveys')->count() < 10) {
            $this->createMockHardwareSurveys();
        }
    }

    /**
     * Create mock users
     */
    private function createMockUsers(): void
    {
        $users = [
            [
                'name' => 'John Doe',
                'email' => 'john@example.com',
                'password' => Hash::make('password'),
                'uuid' => Str::uuid()->toString(),
                'is_admin' => false,
                'is_active' => true,
                'email_verified_at' => now(),
            ],
            [
                'name' => 'Jane Smith',
                'email' => 'jane@example.com',
                'password' => Hash::make('password'),
                'uuid' => Str::uuid()->toString(),
                'is_admin' => false,
                'is_active' => true,
                'email_verified_at' => now(),
            ],
            [
                'name' => 'Alice Johnson',
                'email' => 'alice@example.com',
                'password' => Hash::make('password'),
                'uuid' => Str::uuid()->toString(),
                'is_admin' => false,
                'is_active' => true,
                'email_verified_at' => now(),
            ],
            [
                'name' => 'Bob Williams',
                'email' => 'bob@example.com',
                'password' => Hash::make('password'),
                'uuid' => Str::uuid()->toString(),
                'is_admin' => false,
                'is_active' => true,
                'email_verified_at' => now(),
            ],
            [
                'name' => 'Carol Brown',
                'email' => 'carol@example.com',
                'password' => Hash::make('password'),
                'uuid' => Str::uuid()->toString(),
                'is_admin' => false,
                'is_active' => true,
                'email_verified_at' => now(),
            ],
        ];

        foreach ($users as $user) {
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
        $users = DB::table('users')->where('is_admin', false)->pluck('id')->toArray();

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
        $users = DB::table('users')->where('is_admin', false)->pluck('id')->toArray();

        $statuses = ['pending', 'in_progress', 'resolved', 'closed'];
        $bugReports = [
            [
                'title' => 'Application crashes when scanning system',
                'description' => 'The app crashes when I try to scan my system with more than 3 drives.',
                'status' => $statuses[array_rand($statuses)],
            ],
            [
                'title' => 'Memory optimization not working',
                'description' => 'Memory optimization feature doesn\'t seem to be working on Windows 11.',
                'status' => $statuses[array_rand($statuses)],
            ],
            [
                'title' => 'UI freezes during heavy operations',
                'description' => 'The interface becomes unresponsive during disk cleanup.',
                'status' => $statuses[array_rand($statuses)],
            ],
            [
                'title' => 'Error message when accessing network settings',
                'description' => 'Getting an unhandled exception when trying to optimize network settings.',
                'status' => $statuses[array_rand($statuses)],
            ],
            [
                'title' => 'Can\'t save custom profiles',
                'description' => 'Custom optimization profiles don\'t save properly.',
                'status' => $statuses[array_rand($statuses)],
            ],
            [
                'title' => 'Application hangs on startup',
                'description' => 'Sometimes the application hangs indefinitely when starting up.',
                'status' => $statuses[array_rand($statuses)],
            ],
            [
                'title' => 'CPU optimization not detecting all cores',
                'description' => 'The CPU optimization only detects half of my CPU cores.',
                'status' => $statuses[array_rand($statuses)],
            ],
            [
                'title' => 'Export results feature not working',
                'description' => 'When I try to export optimization results, nothing happens.',
                'status' => $statuses[array_rand($statuses)],
            ],
            [
                'title' => 'Application crashes with specific GPU model',
                'description' => 'The app consistently crashes with my RTX 4090 graphics card.',
                'status' => $statuses[array_rand($statuses)],
            ],
            [
                'title' => 'Settings reset after application update',
                'description' => 'All my settings were reset after the latest update.',
                'status' => $statuses[array_rand($statuses)],
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
        $users = DB::table('users')->where('is_admin', false)->pluck('id')->toArray();

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
