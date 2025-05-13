<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    /**
     * Run the migrations.
     */
    public function up(): void
    {
        Schema::create('download_logs', function (Blueprint $table) {
            $table->id();
            $table->foreignId('user_id')->constrained()->cascadeOnDelete(); // Link to users table
            $table->ipAddress('ip_address')->nullable(); // Store the IP address
            $table->string('platform')->nullable(); // Operating system platform
            $table->boolean('downloaded')->default(false); // Whether the file was actually downloaded
            $table->timestamp('created_at')->useCurrent(); // Only need created_at
            // No updated_at needed for logs
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('download_logs');
    }
};
