<?php

namespace App\Providers;

use App\Events\FeedbackSubmitted;
use App\Listeners\LogFeedbackSubmission;
use App\Listeners\NotifyAdminsAboutFeedback;
use App\Listeners\ProcessFeedbackAnalytics;
use App\Listeners\ProcessWebhooks;
use Illuminate\Auth\Events\Registered;
use Illuminate\Auth\Listeners\SendEmailVerificationNotification;
use Illuminate\Foundation\Support\Providers\EventServiceProvider as ServiceProvider;
use Illuminate\Support\Facades\Event;

class EventServiceProvider extends ServiceProvider
{
    /**
     * The event to listener mappings for the application.
     *
     * @var array<class-string, array<int, class-string>>
     */
    protected $listen = [
        Registered::class => [
            SendEmailVerificationNotification::class,
        ],
        // Feedback events
        FeedbackSubmitted::class => [
            LogFeedbackSubmission::class,
            NotifyAdminsAboutFeedback::class,
            ProcessFeedbackAnalytics::class,
            ProcessWebhooks::class,
        ],
        // Add your other event listeners here
        // \App\Events\ReviewSubmitted::class => [
        //     \App\Listeners\LogReviewSubmission::class,
        //     \App\Listeners\IncrementReviewMetric::class,
        // ],
    ];

    /**
     * Register any events for your application.
     */
    public function boot(): void
    {
        //
    }

    /**
     * Determine if events and listeners should be automatically discovered.
     */
    public function shouldDiscoverEvents(): bool
    {
        return false;
    }
}
