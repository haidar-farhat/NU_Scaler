<?php

namespace App\Http\Controllers\Api\Admin;

use App\Http\Controllers\Controller;
use App\Http\Resources\Api\V1\BugReportResource;
use App\Http\Resources\Api\V1\HardwareSurveyResource;
use App\Http\Resources\Api\V1\ReviewResource;
use App\Models\BugReport;
use App\Models\HardwareSurvey;
use App\Models\Review;
use Illuminate\Http\JsonResponse;
use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;

class FeedbackController extends Controller
{
    /**
     * Display a listing of reviews.
     */
    public function listReviews(Request $request): JsonResponse
    {
        $reviews = Review::query()
            ->when($request->has('rating'), function ($query) use ($request) {
                $query->where('rating', $request->rating);
            })
            ->latest()
            ->paginate(15);

        return ReviewResource::collection($reviews)
            ->additional([
                'meta' => [
                    'average_rating' => Review::avg('rating'),
                    'total_count' => Review::count(),
                ],
            ])
            ->response();
    }

    /**
     * Display a listing of bug reports.
     */
    public function listBugReports(Request $request): JsonResponse
    {
        $bugReports = BugReport::query()
            ->when($request->has('severity'), function ($query) use ($request) {
                $query->where('severity', $request->severity);
            })
            ->latest()
            ->paginate(15);

        return BugReportResource::collection($bugReports)
            ->additional([
                'meta' => [
                    'severity_counts' => $this->getBugReportSeverityCounts(),
                    'total_count' => BugReport::count(),
                ],
            ])
            ->response();
    }

    /**
     * Display a listing of hardware surveys.
     */
    public function listHardwareSurveys(Request $request): JsonResponse
    {
        $hardwareSurveys = HardwareSurvey::query()
            ->when($request->has('os'), function ($query) use ($request) {
                $query->where('os', 'like', '%' . $request->os . '%');
            })
            ->latest()
            ->paginate(15);

        return HardwareSurveyResource::collection($hardwareSurveys)
            ->additional([
                'meta' => [
                    'os_distribution' => $this->getOsDistribution(),
                    'total_count' => HardwareSurvey::count(),
                ],
            ])
            ->response();
    }

    /**
     * Get bug report severity counts.
     */
    private function getBugReportSeverityCounts(): array
    {
        $severityCounts = BugReport::selectRaw('severity, count(*) as count')
            ->groupBy('severity')
            ->get()
            ->pluck('count', 'severity')
            ->toArray();

        return $severityCounts;
    }

    /**
     * Get OS distribution.
     */
    private function getOsDistribution(): array
    {
        $osDistribution = HardwareSurvey::selectRaw('os, count(*) as count')
            ->groupBy('os')
            ->get()
            ->pluck('count', 'os')
            ->toArray();

        return $osDistribution;
    }
} 