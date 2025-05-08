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
        if (!Schema::hasTable('bug_reports')) {
            Schema::create('bug_reports', function (Blueprint $table) {
                $table->id();
                $table->text('description');
                $table->string('category')->nullable();
                $table->string('severity');
                $table->text('steps_to_reproduce')->nullable();
                $table->json('system_info')->nullable();
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
        Schema::dropIfExists('bug_reports');
    }
};
