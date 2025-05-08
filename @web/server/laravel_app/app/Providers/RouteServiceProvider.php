<?php

namespace App\Providers;

use Illuminate\Cache\RateLimiting\Limit;
use Illuminate\Foundation\Support\Providers\RouteServiceProvider as ServiceProvider;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\RateLimiter;
use Illuminate\Support\Facades\Route;

class RouteServiceProvider extends ServiceProvider
{
    /**
     * The path to your application's "home" route.
     *
     * Typically, users are redirected here after authentication.
     *
     * @var string
     */
    public const HOME = '/home'; // Adjust if you have a web home route

    /**
     * Define your route model bindings, pattern filters, and other route configuration.
     */
    public function boot(): void
    {
        $this->configureRateLimiting();

        $this->routes(function () {
            Route::middleware('api')
                ->prefix('api')
                ->group(base_path('routes/api.php'));

            Route::middleware('web')
                ->group(base_path('routes/web.php'));
        });
    }

    /**
     * Configure the rate limiters for the application.
     */
    protected function configureRateLimiting(): void
    {
        // Default API limiter (applied via 'api' middleware group in Kernel)
        RateLimiter::for('api', function (Request $request) {
            // Use authenticated user for rate limiting when available
            $user = $request->user();

            // Rate limits based on API version and user type
            if ($request->is('api/v1/*')) {
                // V1 API has higher limits
                return $user && $user->is_admin
                    ? Limit::perMinute(120)
                    : Limit::perMinute(60)->by($user?->id ?: $request->ip());
            }

            // Admin API endpoints
            if ($request->is('api/admin/*')) {
                return $user && $user->is_admin
                    ? Limit::perMinute(500)
                    : Limit::perMinute(5)->by($request->ip());
            }

            // General API throttling (default)
            return $user
                ? Limit::perMinute(60)->by($user->id)
                : Limit::perMinute(30)->by($request->ip());
        });

        // Stricter limiter for registration attempts
        RateLimiter::for('registrations', function (Request $request) {
            // Limit by IP address: 10 attempts per hour
            return Limit::perHour(10)->by($request->ip());
        });

         // Limiter for feedback submissions (adjust as needed)
        RateLimiter::for('feedback', function (Request $request) {
            // Limit by IP: 100 submissions per hour
            return Limit::perHour(100)->by($request->ip());
        });

         // Limiter for downloads (adjust based on expected usage)
        RateLimiter::for('downloads', function (Request $request) {
            // Limit by authenticated user ID: 5 downloads per hour
            return Limit::perHour(5)->by($request->user()?->id ?: $request->ip());
        });
    }
}
