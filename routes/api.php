<?php

use App\Http\Controllers\Api\V1\BugReportController;
use App\Http\Controllers\Api\V1\HardwareSurveyController;
use App\Http\Controllers\Api\V1\ReviewController;
use App\Http\Controllers\Api\V1\Auth\RegisterController;
use App\Http\Controllers\Api\V1\DownloadController;
use App\Http\Controllers\Api\Admin\FeedbackController;
use App\Http\Controllers\Api\Admin\AuthController;
use App\Http\Controllers\Api\Admin\MetricsController;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Route;

/*
|--------------------------------------------------------------------------
| API Routes
|--------------------------------------------------------------------------
|
| Here is where you can register API routes for your application. These
| routes are loaded by the RouteServiceProvider and all of them will
| be assigned to the "api" middleware group. Make something great!
|
*/

Route::middleware('auth:sanctum')->get('/user', function (Request $request) {
    return $request->user();
});

// Public User Authentication (Version 1)
Route::prefix('v1/auth')->name('api.v1.auth.')->group(function () {
    Route::post('register', [RegisterController::class, 'register'])
         ->middleware('throttle:registrations')
         ->name('register');
    // Add login route for regular users if needed, separate from admin login
    // Route::post('login', [LoginController::class, 'login'])->name('login');
});

// Authenticated User Actions (Version 1)
Route::prefix('v1')->middleware('auth:sanctum')->name('api.v1.')->group(function () {
    Route::get('download', [DownloadController::class, 'getDownloadLink'])
         ->middleware('throttle:downloads') 
         ->name('download');
    // Add other authenticated user routes here (e.g., profile management)
});

// Public Feedback API Endpoints (Version 1)
Route::prefix('v1/feedback')->name('api.v1.feedback.')->group(function () {
    Route::middleware('throttle:feedback')->group(function(){
        Route::post('reviews', [ReviewController::class, 'store'])->name('reviews.store');
        Route::post('bug-reports', [BugReportController::class, 'store'])->name('bug-reports.store');
        Route::post('hardware-surveys', [HardwareSurveyController::class, 'store'])->name('hardware-surveys.store');
    });
});

// Admin API endpoints (Protected with auth and admin middleware)
Route::prefix('admin')->name('api.admin.')->group(function () {
    // Admin Authentication
    Route::post('login', [AuthController::class, 'login'])->name('login');
    // Add logout route later if needed: Route::post('logout', [AuthController::class, 'logout'])->middleware('auth:sanctum')->name('logout');

    // Protected Admin Routes
    Route::middleware(['auth:sanctum', 'is_admin'])->group(function () {
        // Feedback Listing & Detail
        Route::prefix('feedback')->name('feedback.')->group(function () {
            // Reviews
            Route::get('/reviews', [FeedbackController::class, 'listReviews'])->name('reviews.list');
            Route::get('/reviews/{review}', [FeedbackController::class, 'showReview'])->name('reviews.show');
            
            // Bug Reports
            Route::get('/bug-reports', [FeedbackController::class, 'listBugReports'])->name('bug_reports.list');
            Route::get('/bug-reports/{bugReport}', [FeedbackController::class, 'showBugReport'])->name('bug_reports.show');
            
            // Hardware Surveys
            Route::get('/hardware-surveys', [FeedbackController::class, 'listHardwareSurveys'])->name('hardware_surveys.list');
            Route::get('/hardware-surveys/{hardwareSurvey}', [FeedbackController::class, 'showHardwareSurvey'])->name('hardware_surveys.show');
        });

        // Metrics & Analytics
        Route::prefix('metrics')->name('metrics.')->group(function () {
            // Review metrics
            Route::get('reviews-distribution', [MetricsController::class, 'reviewsDistribution'])
                ->name('reviews_distribution');
            
            // Bug report metrics
            Route::get('bug-reports-severity', [MetricsController::class, 'bugReportsSeverity'])
                ->name('bug_reports_severity');
            
            // Hardware survey metrics
            Route::get('hardware-os-distribution', [MetricsController::class, 'hardwareOsDistribution'])
                ->name('hardware_os_distribution');
            Route::get('hardware-gpu-distribution', [MetricsController::class, 'hardwareGpuDistribution'])
                ->name('hardware_gpu_distribution');
            
            // Combined metrics
            Route::get('submission-trends', [MetricsController::class, 'submissionTrends'])
                ->name('submission_trends');
        });
    });
});

// TODO: Remove older public feedback routes if v1 replaces them?
// Route::post('/feedback/review', [App\Http\Controllers\Api\FeedbackController::class, 'submitReview'])->name('api.feedback.review');
// Route::post('/feedback/bug-report', [App\Http\Controllers\Api\FeedbackController::class, 'submitBugReport'])->name('api.feedback.bug_report');
// Route::post('/feedback/hardware-survey', [App\Http\Controllers\Api\FeedbackController::class, 'submitHardwareSurvey'])->name('api.feedback.hardware_survey');

// TODO: Add Download routes
// TODO: Add Admin routes