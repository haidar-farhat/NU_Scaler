<?php

namespace App\Http\Controllers\Api\V1;

use App\Events\FeedbackSubmitted;
use App\Http\Controllers\Controller;
use App\Http\Requests\BugReportRequest;
use App\Http\Requests\HardwareSurveyRequest;
use App\Http\Requests\ReviewRequest;
use App\Models\BugReport;
use App\Models\HardwareSurvey;
use App\Models\Review;
use Illuminate\Http\JsonResponse;

class FeedbackController extends Controller
{
    /**
     * Store a new review.
     *
     * @param ReviewRequest $request
     * @return JsonResponse
     */
    public function storeReview(ReviewRequest $request): JsonResponse
    {
        // Create a new review
        $review = Review::create([
            'rating' => $request->rating,
            'comment' => $request->comment,
            'name' => $request->name,
            'email' => $request->email,
            'user_uuid' => $request->user() ? $request->user()->uuid : null,
        ]);

        // Dispatch the event
        event(new FeedbackSubmitted($review, 'review'));

        return response()->json([
            'message' => 'Review submitted successfully',
            'data' => $review,
        ], 201);
    }

    /**
     * Store a new bug report.
     *
     * @param BugReportRequest $request
     * @return JsonResponse
     */
    public function storeBugReport(BugReportRequest $request): JsonResponse
    {
        // Create a new bug report
        $bugReport = BugReport::create([
            'description' => $request->description,
            'category' => $request->category,
            'severity' => $request->severity,
            'steps_to_reproduce' => $request->steps_to_reproduce,
            'system_info' => $request->system_info,
            'user_uuid' => $request->user() ? $request->user()->uuid : null,
        ]);

        // Dispatch the event
        event(new FeedbackSubmitted($bugReport, 'bug-report'));

        return response()->json([
            'message' => 'Bug report submitted successfully',
            'data' => $bugReport,
        ], 201);
    }

    /**
     * Store a new hardware survey.
     *
     * @param HardwareSurveyRequest $request
     * @return JsonResponse
     */
    public function storeHardwareSurvey(HardwareSurveyRequest $request): JsonResponse
    {
        // Create a new hardware survey
        $hardwareSurvey = HardwareSurvey::create([
            'cpu_model' => $request->cpu_model,
            'gpu_model' => $request->gpu_model,
            'ram_size' => $request->ram_size,
            'os' => $request->os,
            'resolution' => $request->resolution,
            'monitor_refresh_rate' => $request->monitor_refresh_rate,
            'user_uuid' => $request->user() ? $request->user()->uuid : null,
        ]);

        // Dispatch the event
        event(new FeedbackSubmitted($hardwareSurvey, 'hardware-survey'));

        return response()->json([
            'message' => 'Hardware survey submitted successfully',
            'data' => $hardwareSurvey,
        ], 201);
    }
}
