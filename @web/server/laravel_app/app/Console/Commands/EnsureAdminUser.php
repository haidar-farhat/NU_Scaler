<?php

namespace App\Console\Commands;

use Illuminate\Console\Command;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Hash;

class EnsureAdminUser extends Command
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'app:ensure-admin-user';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Ensures that admin@nuscaler.com exists with admin privileges';

    /**
     * Execute the console command.
     */
    public function handle()
    {
        $this->info('Checking for admin user...');

        $admin = DB::table('users')->where('email', 'admin@nuscaler.com')->first();

        if ($admin) {
            $this->info('Admin user exists, ensuring admin privileges are set...');

            // Update the admin user to ensure admin privileges
            DB::table('users')
                ->where('email', 'admin@nuscaler.com')
                ->update([
                    'is_admin' => true,
                    'is_active' => true,
                    'updated_at' => now(),
                ]);

            $this->info('Admin privileges confirmed for admin@nuscaler.com');
        } else {
            $this->info('Admin user does not exist, creating...');

            // Create the admin user
            DB::table('users')->insert([
                'name' => 'Admin',
                'email' => 'admin@nuscaler.com',
                'password' => Hash::make('password'),
                'is_admin' => true,
                'is_active' => true,
                'email_verified_at' => now(),
                'created_at' => now(),
                'updated_at' => now(),
            ]);

            $this->info('Admin user created with email admin@nuscaler.com and password "password"');
        }

        $this->info('Also checking for secondary admin user...');

        $haydar = DB::table('users')->where('email', 'haydar@nuscaler.com')->first();

        if ($haydar) {
            $this->info('Haydar admin user exists, ensuring admin privileges are set...');

            // Update the haydar user to ensure admin privileges
            DB::table('users')
                ->where('email', 'haydar@nuscaler.com')
                ->update([
                    'is_admin' => true,
                    'is_active' => true,
                    'updated_at' => now(),
                ]);

            $this->info('Admin privileges confirmed for haydar@nuscaler.com');
        } else {
            $this->info('Haydar admin user does not exist, creating...');

            // Create the haydar admin user
            DB::table('users')->insert([
                'name' => 'Haydar',
                'email' => 'haydar@nuscaler.com',
                'password' => Hash::make('password'),
                'is_admin' => true,
                'is_active' => true,
                'email_verified_at' => now(),
                'created_at' => now(),
                'updated_at' => now(),
            ]);

            $this->info('Haydar admin user created with email haydar@nuscaler.com and password "password"');
        }

        $this->info('Admin users setup completed successfully.');
    }
}
