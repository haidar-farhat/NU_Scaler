<?php

namespace App\Providers;

use App\Models\BugReport;
use App\Models\HardwareSurvey;
use App\Models\Review;
use App\Observers\BugReportObserver;
use App\Observers\HardwareSurveyObserver;
use App\Observers\ReviewObserver;
use Illuminate\Support\ServiceProvider;

class AppServiceProvider extends ServiceProvider
{
    /**
     * Register any application services.
     */
    public function register(): void
    {
        //
    }

    /**
     * Bootstrap any application services.
     */
    public function boot(): void
    {
        Review::observe(ReviewObserver::class);
        BugReport::observe(BugReportObserver::class);
        HardwareSurvey::observe(HardwareSurveyObserver::class);
    }
} 