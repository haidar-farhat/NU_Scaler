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
        if (!Schema::hasTable('hardware_surveys')) {
            Schema::create('hardware_surveys', function (Blueprint $table) {
                $table->id();
                $table->string('cpu_model');
                $table->string('gpu_model');
                $table->integer('ram_size');
                $table->string('os');
                $table->string('resolution');
                $table->integer('monitor_refresh_rate')->nullable();
                $table->text('additional_info')->nullable();
                $table->uuid('user_uuid')->nullable();
                $table->timestamps();
            });
        }
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('hardware_surveys');
    }
};
