<?php

namespace App\Http\Controllers\Api\Admin;

use App\Http\Controllers\Controller;
use App\Models\BugReport;
use App\Models\HardwareSurvey;
use App\Models\Review;
use Illuminate\Http\JsonResponse;
use Illuminate\Http\Request;

class AdminFeedbackController extends Controller
{
    /**
     * Display a listing of reviews.
     *
     * @param Request $request
     * @return JsonResponse
     */
    public function index(Request $request): JsonResponse
    {
        // Get reviews with optional filters
        $reviews = Review::query()
            ->when($request->filled('rating'), function ($query) use ($request) {
                return $query->where('rating', $request->rating);
            })
            ->when($request->filled('search'), function ($query) use ($request) {
                return $query->where('comment', 'like', '%' . $request->search . '%');
            })
            ->when($request->filled('from_date'), function ($query) use ($request) {
                return $query->whereDate('created_at', '>=', $request->from_date);
            })
            ->when($request->filled('to_date'), function ($query) use ($request) {
                return $query->whereDate('created_at', '<=', $request->to_date);
            })
            ->latest()
            ->paginate($request->per_page ?? 15);

        return response()->json($reviews);
    }

    /**
     * Display the specified review.
     *
     * @param Review $review
     * @return JsonResponse
     */
    public function show(Review $review): JsonResponse
    {
        return response()->json(['data' => $review]);
    }

    /**
     * Display a listing of bug reports.
     *
     * @param Request $request
     * @return JsonResponse
     */
    public function indexBugReports(Request $request): JsonResponse
    {
        // Get bug reports with optional filters
        $bugReports = BugReport::query()
            ->when($request->filled('severity'), function ($query) use ($request) {
                return $query->where('severity', $request->severity);
            })
            ->when($request->filled('category'), function ($query) use ($request) {
                return $query->where('category', $request->category);
            })
            ->when($request->filled('search'), function ($query) use ($request) {
                return $query->where('description', 'like', '%' . $request->search . '%');
            })
            ->when($request->filled('from_date'), function ($query) use ($request) {
                return $query->whereDate('created_at', '>=', $request->from_date);
            })
            ->when($request->filled('to_date'), function ($query) use ($request) {
                return $query->whereDate('created_at', '<=', $request->to_date);
            })
            ->latest()
            ->paginate($request->per_page ?? 15);

        return response()->json($bugReports);
    }

    /**
     * Display the specified bug report.
     *
     * @param BugReport $bugReport
     * @return JsonResponse
     */
    public function showBugReport(BugReport $bugReport): JsonResponse
    {
        return response()->json(['data' => $bugReport]);
    }

    /**
     * Display a listing of hardware surveys.
     *
     * @param Request $request
     * @return JsonResponse
     */
    public function indexHardwareSurveys(Request $request): JsonResponse
    {
        // Get hardware surveys with optional filters
        $hardwareSurveys = HardwareSurvey::query()
            ->when($request->filled('os'), function ($query) use ($request) {
                return $query->where('os', 'like', '%' . $request->os . '%');
            })
            ->when($request->filled('gpu_model'), function ($query) use ($request) {
                return $query->where('gpu_model', 'like', '%' . $request->gpu_model . '%');
            })
            ->when($request->filled('cpu_model'), function ($query) use ($request) {
                return $query->where('cpu_model', 'like', '%' . $request->cpu_model . '%');
            })
            ->when($request->filled('min_ram'), function ($query) use ($request) {
                return $query->where('ram_size', '>=', $request->min_ram);
            })
            ->when($request->filled('from_date'), function ($query) use ($request) {
                return $query->whereDate('created_at', '>=', $request->from_date);
            })
            ->when($request->filled('to_date'), function ($query) use ($request) {
                return $query->whereDate('created_at', '<=', $request->to_date);
            })
            ->latest()
            ->paginate($request->per_page ?? 15);

        return response()->json($hardwareSurveys);
    }

    /**
     * Display the specified hardware survey.
     *
     * @param HardwareSurvey $hardwareSurvey
     * @return JsonResponse
     */
    public function showHardwareSurvey(HardwareSurvey $hardwareSurvey): JsonResponse
    {
        return response()->json(['data' => $hardwareSurvey]);
    }

    /**
     * Remove the specified review.
     *
     * @param Review $review
     * @return JsonResponse
     */
    public function destroyReview(Review $review): JsonResponse
    {
        $review->delete();
        return response()->json(['message' => 'Review deleted successfully']);
    }

    /**
     * Remove the specified bug report.
     *
     * @param BugReport $bugReport
     * @return JsonResponse
     */
    public function destroyBugReport(BugReport $bugReport): JsonResponse
    {
        $bugReport->delete();
        return response()->json(['message' => 'Bug report deleted successfully']);
    }

    /**
     * Remove the specified hardware survey.
     *
     * @param HardwareSurvey $hardwareSurvey
     * @return JsonResponse
     */
    public function destroyHardwareSurvey(HardwareSurvey $hardwareSurvey): JsonResponse
    {
        $hardwareSurvey->delete();
        return response()->json(['message' => 'Hardware survey deleted successfully']);
    }
}
