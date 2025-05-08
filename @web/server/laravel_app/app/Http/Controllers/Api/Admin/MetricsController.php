<?php

namespace App\Http\Controllers\Api\Admin;

use App\Http\Controllers\Controller;
use App\Models\BugReport;
use App\Models\Review;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB; // Import DB facade for aggregate queries
use Illuminate\Http\JsonResponse;

class MetricsController extends Controller
{
    /**
     * Get the distribution of review ratings.
     *
     * @param Request $request
     * @return JsonResponse
     */
    public function reviewsDistribution(Request $request): JsonResponse
    {
        $distribution = Review::query()
            ->select('rating', DB::raw('count(*) as count'))
            ->groupBy('rating')
            ->orderBy('rating')
            ->get(); // Returns collection like [{rating: 1, count: 5}, {rating: 2, count: 10}]

        // Optional: Format for specific charting libraries if needed
        // Example: Format for Chart.js labels/data
        // $labels = $distribution->pluck('rating');
        // $data = $distribution->pluck('count');
        // return response()->json(['labels' => $labels, 'data' => $data]);

        return response()->json($distribution);
    }

    /**
     * Get the count of bug reports by severity.
     *
     * @param Request $request
     * @return JsonResponse
     */
    public function bugReportsSeverity(Request $request): JsonResponse
    {
        $severityCounts = BugReport::query()
            ->select('severity', DB::raw('count(*) as count'))
            ->groupBy('severity')
            ->orderByRaw("CASE severity
                             WHEN 'critical' THEN 1
                             WHEN 'high' THEN 2
                             WHEN 'medium' THEN 3
                             WHEN 'low' THEN 4
                             ELSE 5 END") // Order by severity level
            ->get();

        return response()->json($severityCounts);
    }

    // Add methods for other metrics here
}
