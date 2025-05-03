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

// Apply general rate limiting to all API routes
Route::middleware(['api.rate.limit:public'])->group(function () {

    // User info endpoint (requires authentication)
    Route::middleware('auth:sanctum')->get('/user', function (Request $request) {
        return $request->user();
    });
    
    // Public User Authentication (Version 1)
    Route::prefix('v1/auth')->name('api.v1.auth.')->group(function () {
        Route::post('register', [RegisterController::class, 'register'])
             ->middleware('api.rate.limit:register')
             ->name('register');
        // Add login route for regular users if needed, separate from admin login
        // Route::post('login', [LoginController::class, 'login'])->middleware('api.rate.limit:login')->name('login');
    });
    
    // Authenticated User Actions (Version 1)
    Route::prefix('v1')->middleware('auth:sanctum')->name('api.v1.')->group(function () {
        Route::get('download', [DownloadController::class, 'getDownloadLink'])
             ->middleware('api.rate.limit:downloads') 
             ->name('download');
        // Add other authenticated user routes here (e.g., profile management)
    });
    
    // Public Feedback API Endpoints (Version 1)
    Route::prefix('v1/feedback')->name('api.v1.feedback.')->group(function () {
        Route::middleware('api.rate.limit:feedback')->group(function() {
            Route::post('reviews', [ReviewController::class, 'store'])
                 ->middleware('api.rate.limit:reviews')
                 ->name('reviews.store');
                 
            Route::post('bug-reports', [BugReportController::class, 'store'])
                 ->middleware('api.rate.limit:bug_reports')
                 ->name('bug-reports.store');
                 
            Route::post('hardware-surveys', [HardwareSurveyController::class, 'store'])
                 ->middleware('api.rate.limit:hardware_surveys')
                 ->name('hardware-surveys.store');
        });
    });
    
    // Admin API endpoints (Protected with auth and admin middleware)
    Route::prefix('admin')->name('api.admin.')->group(function () {
        // Admin Authentication
        Route::post('login', [AuthController::class, 'login'])
             ->middleware('api.rate.limit:login')
             ->name('login');
        // Add logout route later if needed
    
        // Protected Admin Routes
        Route::middleware(['auth:sanctum', 'is_admin', 'api.rate.limit:admin'])->group(function () {
            // Feedback Listing & Detail
            Route::prefix('feedback')->name('feedback.')->group(function () {
                // Reviews
                Route::get('/reviews', [FeedbackController::class, 'listReviews'])
                     ->name('reviews.list')
                     ->middleware('can:viewAny,App\Models\Review');
                     
                Route::get('/reviews/{review}', [FeedbackController::class, 'showReview'])
                     ->name('reviews.show')
                     ->middleware('can:view,review');
                
                // Bug Reports
                Route::get('/bug-reports', [FeedbackController::class, 'listBugReports'])
                     ->name('bug_reports.list')
                     ->middleware('can:viewAny,App\Models\BugReport');
                     
                Route::get('/bug-reports/{bugReport}', [FeedbackController::class, 'showBugReport'])
                     ->name('bug_reports.show')
                     ->middleware('can:view,bugReport');
                
                // Hardware Surveys
                Route::get('/hardware-surveys', [FeedbackController::class, 'listHardwareSurveys'])
                     ->name('hardware_surveys.list')
                     ->middleware('can:viewAny,App\Models\HardwareSurvey');
                     
                Route::get('/hardware-surveys/{hardwareSurvey}', [FeedbackController::class, 'showHardwareSurvey'])
                     ->name('hardware_surveys.show')
                     ->middleware('can:view,hardwareSurvey');
            });
    
            // Metrics & Analytics
            Route::prefix('metrics')->name('metrics.')->middleware('api.rate.limit:metrics')->group(function () {
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
                    
                // Advanced analytics
                Route::get('active-feedback-users', [MetricsController::class, 'activeFeedbackUsers'])
                    ->name('active_feedback_users');
                    
                Route::get('feedback-correlation', [MetricsController::class, 'feedbackCorrelation'])
                    ->name('feedback_correlation');
            });
        });
    });
});

// TODO: Remove older public feedback routes if v1 replaces them?
// Route::post('/feedback/review', [App\Http\Controllers\Api\FeedbackController::class, 'submitReview'])->name('api.feedback.review');
// Route::post('/feedback/bug-report', [App\Http\Controllers\Api\FeedbackController::class, 'submitBugReport'])->name('api.feedback.bug_report');
// Route::post('/feedback/hardware-survey', [App\Http\Controllers\Api\FeedbackController::class, 'submitHardwareSurvey'])->name('api.feedback.hardware_survey');

// TODO: Add Download routes
// TODO: Add Admin routes