<?php

namespace App\Providers;

use App\Models\BugReport;
use App\Models\HardwareSurvey;
use App\Models\Review;
use App\Policies\BugReportPolicy;
use App\Policies\HardwareSurveyPolicy;
use App\Policies\ReviewPolicy;
use Illuminate\Foundation\Support\Providers\AuthServiceProvider as ServiceProvider;
use Illuminate\Support\Facades\Gate;

class AuthServiceProvider extends ServiceProvider
{
    /**
     * The model to policy mappings for the application.
     *
     * @var array<class-string, class-string>
     */
    protected $policies = [
        Review::class => ReviewPolicy::class,
        BugReport::class => BugReportPolicy::class,
        HardwareSurvey::class => HardwareSurveyPolicy::class,
    ];

    /**
     * Register any authentication / authorization services.
     */
    public function boot(): void
    {
        $this->registerPolicies();

        // Define a gate for checking admin status
        Gate::define('admin', function ($user) {
            return $user && $user->is_admin;
        });
    }
} 