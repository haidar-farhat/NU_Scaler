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
        Schema::create('bug_reports', function (Blueprint $table) {
            $table->id();
            $table->text('description');
            $table->string('log_path')->nullable(); // Optional path to uploaded log file
            $table->enum('severity', ['low', 'medium', 'high', 'critical']);
            $table->json('system_info')->nullable(); // CPU, GPU, RAM, OS etc.
            $table->foreignUuid('user_uuid')->nullable()->constrained('users', 'uuid')->nullOnDelete(); // Link to user (optional)
            $table->timestamps();
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('bug_reports');
    }
}; 