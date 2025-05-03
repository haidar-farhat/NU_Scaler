<?php

use Illuminate\Http\Request;
use Illuminate\Support\Facades\Route;
use App\Http\Controllers\Api\V1\AuthController;
use App\Http\Controllers\Api\V1\FeedbackController;
use App\Http\Controllers\Api\V1\DownloadController;
use App\Http\Controllers\Api\V1\WebhookController;
use App\Http\Controllers\Api\Admin\AdminFeedbackController;
use App\Http\Controllers\Api\Admin\AdminMetricsController;
use App\Http\Controllers\Api\Admin\AdminAuthController; // Assuming separate admin auth controller
use App\Http\Controllers\Api\Admin\LogDashboardController;

/*
|--------------------------------------------------------------------------
| API Routes
|--------------------------------------------------------------------------
|
| Here is where you can register API routes for your application. These
| routes are loaded by the RouteServiceProvider within a group which
| is assigned the "api" middleware group and prefixed with "/api". Enjoy building your API!
|
*/

// Public Routes (V1 - Assuming versioning)
Route::prefix('v1')->group(function () {
    // Authentication - Updated to match test expectations
    Route::post('/auth/register', [AuthController::class, 'register'])->name('api.v1.register');
    Route::post('/login', [AuthController::class, 'login'])->name('api.v1.login');
    Route::post('/logout', [AuthController::class, 'logout'])->middleware('auth:sanctum')->name('api.v1.logout');

    // Feedback Submission - Public with caching for analytics
    Route::middleware('cache.response:300')->group(function () {
        Route::get('/feedback/stats', [FeedbackController::class, 'publicStats'])->name('api.v1.feedback.stats');
    });

    // Feedback submission endpoints - Not cached because they're POST requests
    Route::post('/feedback/reviews', [FeedbackController::class, 'storeReview'])->name('api.v1.feedback.reviews.store');
    Route::post('/feedback/bug-reports', [FeedbackController::class, 'storeBugReport'])->name('api.v1.feedback.bug-reports.store');
    Route::post('/feedback/hardware-surveys', [FeedbackController::class, 'storeHardwareSurvey'])->name('api.v1.feedback.hardware-surveys.store');

    // Download Info (Protected by auth)
    Route::get('/download', [DownloadController::class, 'getDownloadLink'])->middleware('auth:sanctum')->name('api.v1.download');

    // Route to get authenticated user info
    Route::middleware('auth:sanctum')->get('/user', function (Request $request) {
        return $request->user();
    })->name('api.v1.user');
});

// Authenticated User Actions (Version 1)
Route::prefix('v1')->middleware('api.secured')->name('api.v1.')->group(function () {
    Route::get('download', [DownloadController::class, 'getDownloadLink'])
         ->middleware('api.rate.limit:downloads')
         ->name('download');

    // Webhook endpoints
    Route::prefix('webhooks')->name('webhooks.')->group(function () {
        Route::get('/', [WebhookController::class, 'index'])->name('index');
        Route::post('/', [WebhookController::class, 'store'])->name('store');
        Route::get('/{webhook}', [WebhookController::class, 'show'])->name('show');
        Route::put('/{webhook}', [WebhookController::class, 'update'])->name('update');
        Route::delete('/{webhook}', [WebhookController::class, 'destroy'])->name('destroy');
        Route::get('/{webhook}/logs', [WebhookController::class, 'logs'])->name('logs');
        Route::post('/{webhook}/regenerate-secret', [WebhookController::class, 'regenerateSecret'])->name('regenerate-secret');
        Route::post('/{webhook}/test', [WebhookController::class, 'test'])->name('test');
        Route::post('/logs/{log}/retry', [WebhookController::class, 'retry'])->name('retry');
    });

    // Add other authenticated user routes here (e.g., profile management)
});

// Admin Routes - Using full middleware class reference to avoid alias issues
Route::prefix('admin')->name('api.admin.')
    ->middleware(['auth:sanctum', \App\Http\Middleware\IsAdmin::class])
    ->group(function () {
        // Admin Auth (Login might be separate or handled differently)
        // The test hits /login/admin - let's map it, assuming a dedicated controller/method
        // Note: The login route itself shouldn't typically require auth middleware
        Route::post('/login', [AdminAuthController::class, 'login'])
            ->name('login')
            ->withoutMiddleware(['auth:sanctum', \App\Http\Middleware\IsAdmin::class]);

        // Feedback Management
        Route::get('/reviews', [AdminFeedbackController::class, 'index'])->name('reviews.index');
        Route::get('/reviews/{review}', [AdminFeedbackController::class, 'show'])->name('reviews.show');
        Route::get('/bug-reports', [AdminFeedbackController::class, 'indexBugReports'])->name('bug-reports.index');
        Route::get('/bug-reports/{bugReport}', [AdminFeedbackController::class, 'showBugReport'])->name('bug-reports.show');
        Route::get('/hardware-surveys', [AdminFeedbackController::class, 'indexHardwareSurveys'])->name('hardware-surveys.index');
        Route::get('/hardware-surveys/{hardwareSurvey}', [AdminFeedbackController::class, 'showHardwareSurvey'])->name('hardware-surveys.show');

        // Metrics and Analytics
        Route::get('/metrics/dashboard', [AdminMetricsController::class, 'dashboard'])->name('metrics.dashboard');
        Route::get('/metrics/reviews', [AdminMetricsController::class, 'reviewMetrics'])->name('metrics.reviews');
        Route::get('/metrics/bug-reports', [AdminMetricsController::class, 'bugReportMetrics'])->name('metrics.bug-reports');
        Route::get('/metrics/hardware-surveys', [AdminMetricsController::class, 'hardwareSurveyMetrics'])->name('metrics.hardware-surveys');
        Route::get('/metrics/user-growth', [AdminMetricsController::class, 'userGrowthTrends'])->name('metrics.user-growth');
        Route::get('/metrics/feedback-trends', [AdminMetricsController::class, 'feedbackTrends'])->name('metrics.feedback-trends');
        Route::get('/metrics/export', [AdminMetricsController::class, 'exportAllMetrics'])->name('metrics.export');

        // Log Dashboard
        Route::prefix('logs')->name('logs.')->group(function () {
            Route::get('/', [LogDashboardController::class, 'index'])->name('index');
            Route::get('/stats', [LogDashboardController::class, 'stats'])->name('stats');
            Route::get('/search', [LogDashboardController::class, 'search'])->name('search');
            Route::get('/type/{type}', [LogDashboardController::class, 'listFiles'])->name('list');
            Route::get('/file/{filename}', [LogDashboardController::class, 'show'])->name('show');
            Route::delete('/file/{filename}', [LogDashboardController::class, 'destroy'])->name('destroy');
        });
    });

// Protected Admin Routes
Route::middleware(['auth:sanctum', 'is_admin', 'api.rate.limit:admin'])->group(function () {
    // Replace with the following for enhanced security:
    // Route::middleware(['api.secured', 'is_admin', 'api.rate.limit:admin'])->group(function () {

    // Admin Routes....
});

// Fallback route for unmatched API requests (optional but good practice)
Route::fallback(function(){
    return response()->json(['message' => 'Not Found.'], 404);
})->name('api.fallback.404');
