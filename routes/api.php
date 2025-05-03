<?php

use App\Http\Controllers\Api\V1\BugReportController;
use App\Http\Controllers\Api\V1\HardwareSurveyController;
use App\Http\Controllers\Api\V1\ReviewController;
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

// Public Feedback API Endpoints (Version 1)
Route::prefix('v1/feedback')->group(function () {
    Route::post('reviews', [ReviewController::class, 'store'])->name('api.v1.feedback.reviews.store');
    Route::post('bug-reports', [BugReportController::class, 'store'])->name('api.v1.feedback.bug-reports.store');
    Route::post('hardware-surveys', [HardwareSurveyController::class, 'store'])->name('api.v1.feedback.hardware-surveys.store');
});

// Feedback API endpoints (Protected, assuming admin middleware later)
Route::prefix('admin')->middleware(['auth:sanctum', 'admin'])->group(function () { // Placeholder for admin middleware
    Route::get('/reviews', [App\Http\Controllers\Api\Admin\FeedbackController::class, 'listReviews'])->name('api.admin.reviews.list');
    Route::get('/bug-reports', [App\Http\Controllers\Api\Admin\FeedbackController::class, 'listBugReports'])->name('api.admin.bug_reports.list');
    Route::get('/hardware-surveys', [App\Http\Controllers\Api\Admin\FeedbackController::class, 'listHardwareSurveys'])->name('api.admin.hardware_surveys.list');
});

// Public Feedback Submission - Keep existing or add new ones if needed
Route::post('/feedback/review', [App\Http\Controllers\Api\FeedbackController::class, 'submitReview'])->name('api.feedback.review');
Route::post('/feedback/bug-report', [App\Http\Controllers\Api\FeedbackController::class, 'submitBugReport'])->name('api.feedback.bug_report');
Route::post('/feedback/hardware-survey', [App\Http\Controllers\Api\FeedbackController::class, 'submitHardwareSurvey'])->name('api.feedback.hardware_survey');

// TODO: Add Auth routes (register, login)
// TODO: Add Download routes
// TODO: Add Admin routes 