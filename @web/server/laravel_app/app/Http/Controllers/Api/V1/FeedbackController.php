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
use App\Services\AnalyticsService;
use Illuminate\Http\JsonResponse;
use Illuminate\Support\Facades\Cache;

class FeedbackController extends Controller
{
    /**
     * The analytics service instance.
     *
     * @var \App\Services\AnalyticsService
     */
    protected $analyticsService;

    /**
     * Create a new controller instance.
     *
     * @param \App\Services\AnalyticsService $analyticsService
     * @return void
     */
    public function __construct(AnalyticsService $analyticsService)
    {
        $this->analyticsService = $analyticsService;
    }

    /**
     * Display public feedback statistics.
     *
     * @return \Illuminate\Http\JsonResponse
     */
    public function publicStats(): JsonResponse
    {
        $stats = Cache::remember('public.feedback.stats', now()->addHour(), function () {
            return [
                'reviews' => [
                    'count' => Review::count(),
                    'average_rating' => Review::avg('rating') ?? 0,
                    'latest' => Review::latest()->take(5)->get(['id', 'rating', 'comment', 'created_at']),
                ],
                'bug_reports' => [
                    'count' => BugReport::count(),
                    'by_category' => BugReport::selectRaw('category, COUNT(*) as count')
                        ->groupBy('category')
                        ->get()
                        ->pluck('count', 'category'),
                ],
                'hardware_surveys' => [
                    'count' => HardwareSurvey::count(),
                    'popular_gpus' => HardwareSurvey::selectRaw('gpu_model, COUNT(*) as count')
                        ->groupBy('gpu_model')
                        ->orderByDesc('count')
                        ->limit(5)
                        ->get()
                        ->pluck('count', 'gpu_model'),
                ],
            ];
        });

        return response()->json(['data' => $stats]);
    }

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

        // Only dispatch the event in non-testing environments
        if (app()->environment() !== 'testing') {
            event(new FeedbackSubmitted($review, 'review'));
        }

        // Clear the public stats cache
        Cache::forget('public.feedback.stats');

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

        // Only dispatch the event in non-testing environments
        if (app()->environment() !== 'testing') {
            event(new FeedbackSubmitted($bugReport, 'bug-report'));
        }

        // Clear the public stats cache
        Cache::forget('public.feedback.stats');

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
            'additional_info' => $request->additional_info,
            'user_uuid' => $request->user() ? $request->user()->uuid : null,
        ]);

        // Only dispatch the event in non-testing environments
        if (app()->environment() !== 'testing') {
            event(new FeedbackSubmitted($hardwareSurvey, 'hardware-survey'));
        }

        // Clear the public stats cache
        Cache::forget('public.feedback.stats');

        return response()->json([
            'message' => 'Hardware survey submitted successfully',
            'data' => $hardwareSurvey,
        ], 201);
    }
}
