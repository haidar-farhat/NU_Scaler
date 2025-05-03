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
    // Authentication
    Route::post('/register', [AuthController::class, 'register'])->name('api.v1.register');
    // Assuming a standard login route exists, though not explicitly in failing tests
    Route::post('/login', [AuthController::class, 'login'])->name('api.v1.login');
    Route::post('/logout', [AuthController::class, 'logout'])->middleware('auth:sanctum')->name('api.v1.logout'); // Needs auth

    // Feedback Submission
    Route::post('/feedback', [FeedbackController::class, 'store'])->name('api.v1.feedback.store'); // Public submission

    // Download Info (Protected by auth)
    Route::get('/download-info', [DownloadController::class, 'show'])->middleware('auth:sanctum')->name('api.v1.download.info');

    // Route to get authenticated user info
    Route::middleware('auth:sanctum')->get('/user', function (Request $request) {
        return $request->user();
    })->name('api.v1.user');
});


// Admin Routes
// Prefixing with 'admin' - Note: The final URL will be /api/admin/...
Route::prefix('admin')->name('api.admin.')->middleware(['auth:sanctum', 'is_admin']) // Corrected middleware alias to 'is_admin'
    ->group(function () {
        // Admin Auth (Login might be separate or handled differently)
        // The test hits /login/admin - let's map it, assuming a dedicated controller/method
        // Note: The login route itself shouldn't typically require auth middleware
        Route::post('/login', [AdminAuthController::class, 'login'])->name('login')->withoutMiddleware(['auth:sanctum', 'is_admin']);

        // Feedback Management
        Route::get('/reviews', [AdminFeedbackController::class, 'index'])->name('reviews.index');
        // Add other admin feedback routes (show, delete?) as needed

        // Metrics
        Route::get('/metrics/reviews-distribution', [AdminMetricsController::class, 'reviewsDistribution'])->name('metrics.reviews');
});

// Fallback route for unmatched API requests (optional but good practice)
Route::fallback(function(){
    return response()->json(['message' => 'Not Found.'], 404);
})->name('api.fallback.404');
