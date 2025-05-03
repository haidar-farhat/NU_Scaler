<?php

use Illuminate\Http\Request;
use Illuminate\Support\Facades\Route;
use App\Http\Controllers\Api\V1\AuthController;
use App\Http\Controllers\Api\V1\FeedbackController;
use App\Http\Controllers\Api\V1\DownloadController;
use App\Http\Controllers\Api\Admin\AdminFeedbackController;
use App\Http\Controllers\Api\Admin\AdminMetricsController;
use App\Http\Controllers\Api\Admin\AdminAuthController; // Assuming separate admin auth controller

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

    // Feedback Submission - Updated to match test expectations
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
        // Add other admin feedback routes (show, delete?) as needed

        // Metrics
        Route::get('/metrics/reviews-distribution', [AdminMetricsController::class, 'reviewsDistribution'])->name('metrics.reviews');
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
