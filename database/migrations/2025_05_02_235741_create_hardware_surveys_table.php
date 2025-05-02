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
        Schema::create('hardware_surveys', function (Blueprint $table) {
            $table->id();
            $table->string('cpu')->nullable();
            $table->string('gpu')->nullable();
            $table->integer('ram_gb')->unsigned()->nullable();
            $table->string('os')->nullable();
            $table->string('resolution')->nullable(); // e.g., '1920x1080'
            $table->foreignUuid('user_uuid')->nullable()->constrained('users', 'uuid')->nullOnDelete(); // Link to user (optional)
            $table->timestamps();
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('hardware_surveys');
    }
}; 