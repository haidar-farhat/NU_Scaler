<?php

namespace App\Http\Controllers\Api\V1;

use App\Http\Controllers\Controller;
use Illuminate\Http\Request;

class FeedbackController extends Controller
{
    /**
     * Store a new review.
     *
     * @param  \Illuminate\Http\Request  $request
     * @return \Illuminate\Http\JsonResponse
     */
    public function storeReview(Request $request)
    {
        $request->validate([
            'rating' => 'required|integer|min:1|max:5',
            'comment' => 'required|string|max:1000',
            'name' => 'nullable|string|max:255',
            'email' => 'nullable|email|max:255',
        ]);

        // TODO: Implement actual review storage

        return response()->json([
            'message' => 'Review submitted successfully',
            'review' => [
                'id' => 1, // This would be the actual ID in a real implementation
                'rating' => $request->rating,
                'comment' => $request->comment,
            ],
        ], 201);
    }

    /**
     * Store a new bug report.
     *
     * @param  \Illuminate\Http\Request  $request
     * @return \Illuminate\Http\JsonResponse
     */
    public function storeBugReport(Request $request)
    {
        $request->validate([
            'description' => 'required|string|max:2000',
            'severity' => 'required|string|in:low,medium,high,critical',
            'system_info' => 'required|array',
        ]);

        // TODO: Implement actual bug report storage

        return response()->json([
            'message' => 'Bug report submitted successfully',
            'bug_report' => [
                'id' => 1, // This would be the actual ID in a real implementation
                'description' => $request->description,
                'severity' => $request->severity,
            ],
        ], 201);
    }

    /**
     * Store a new hardware survey.
     *
     * @param  \Illuminate\Http\Request  $request
     * @return \Illuminate\Http\JsonResponse
     */
    public function storeHardwareSurvey(Request $request)
    {
        $request->validate([
            'cpu' => 'required|string|max:255',
            'gpu' => 'required|string|max:255',
            'ram_gb' => 'required|integer|min:1',
            'os' => 'required|string|max:255',
            'resolution' => 'required|string|max:50',
        ]);

        // TODO: Implement actual hardware survey storage

        return response()->json([
            'message' => 'Hardware survey submitted successfully',
            'hardware_survey' => [
                'id' => 1, // This would be the actual ID in a real implementation
                'cpu' => $request->cpu,
                'gpu' => $request->gpu,
                'ram_gb' => $request->ram_gb,
                'os' => $request->os,
                'resolution' => $request->resolution,
            ],
        ], 201);
    }
}
