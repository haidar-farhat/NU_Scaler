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
        // Only create admin users - don't try to create other related data yet
        $defaultAdmins = [
            [
                'name' => 'Admin',
                'email' => 'admin@nuscaler.com',
                'password' => Hash::make('password'),
                'is_admin' => true,
                'is_active' => true,
                'email_verified_at' => now(),
                'created_at' => now(),
                'updated_at' => now(),
            ],
            [
                'name' => 'Haydar',
                'email' => 'haydar@nuscaler.com',
                'password' => Hash::make('password'),
                'is_admin' => true,
                'is_active' => true,
                'email_verified_at' => now(),
                'created_at' => now(),
                'updated_at' => now(),
            ]
        ];

        foreach ($defaultAdmins as $admin) {
            if (!DB::table('users')->where('email', $admin['email'])->exists()) {
                DB::table('users')->insert($admin);
            } else {
                // Update existing user to ensure they are admin
                DB::table('users')
                    ->where('email', $admin['email'])
                    ->update([
                        'is_admin' => true,
                        'is_active' => true,
                        'updated_at' => now(),
                    ]);
            }
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
