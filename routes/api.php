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

Route::middleware(['auth:sanctum'])->get('/user', function (Request $request) {
    return $request->user();
});

// Public Feedback API Endpoints (Version 1)
Route::prefix('v1/feedback')->group(function () {
    Route::post('reviews', [ReviewController::class, 'store'])->name('api.v1.feedback.reviews.store');
    Route::post('bug-reports', [BugReportController::class, 'store'])->name('api.v1.feedback.bug-reports.store');
    Route::post('hardware-surveys', [HardwareSurveyController::class, 'store'])->name('api.v1.feedback.hardware-surveys.store');
});

// TODO: Add Auth routes (register, login)
// TODO: Add Download routes
// TODO: Add Admin routes 