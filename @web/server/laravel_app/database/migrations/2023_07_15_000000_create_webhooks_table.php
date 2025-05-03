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
        Schema::create('webhooks', function (Blueprint $table) {
            $table->id();
            $table->string('name');
            $table->text('url');
            $table->text('description')->nullable();
            $table->boolean('is_active')->default(true);
            $table->json('events')->default('[]');
            $table->string('secret', 100)->nullable();
            $table->foreignId('user_id')->constrained()->onDelete('cascade');
            $table->json('headers')->nullable();
            $table->timestamp('last_triggered_at')->nullable();
            $table->unsignedInteger('fails_count')->default(0);
            $table->timestamps();
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('webhooks');
    }
};
