<?php

namespace App\Http\Controllers\Api\Admin;

use App\Http\Controllers\Controller;
use App\Models\BugReport;
use App\Models\HardwareSurvey;
use App\Models\Review;
use Illuminate\Http\Request;
// Remove JsonResponse, Resources handle the response structure
// use Illuminate\Http\JsonResponse;
use App\Http\Resources\ReviewResource;
use App\Http\Resources\BugReportResource;
use App\Http\Resources\HardwareSurveyResource;
use Illuminate\Http\Resources\Json\AnonymousResourceCollection; // Type hint for resource collections

class FeedbackController extends Controller
{
    /**
     * Display a paginated listing of the reviews.
     */
    public function listReviews(Request $request): AnonymousResourceCollection
    {
        $perPage = $request->query('per_page', 15);
        $filters = $request->query(); // Get all query parameters for filtering

        $reviews = Review::latest()
                         ->filter($filters) // Apply filters from model scope
                         ->paginate($perPage);

        return ReviewResource::collection($reviews->withQueryString()); // Append query string to pagination links
    }

    /**
     * Display a paginated listing of the bug reports.
     */
    public function listBugReports(Request $request): AnonymousResourceCollection
    {
        $perPage = $request->query('per_page', 15);
        $filters = $request->only(['severity']); // Only allow filtering by severity for now

        $bugReports = BugReport::latest()
                             ->filter($filters)
                             ->paginate($perPage);

        return BugReportResource::collection($bugReports->withQueryString());
    }

    /**
     * Display a paginated listing of the hardware surveys.
     */
    public function listHardwareSurveys(Request $request): AnonymousResourceCollection
    {
        $perPage = $request->query('per_page', 15);
        $filters = $request->only(['gpu', 'os']); // Allow filtering by gpu and os

        $surveys = HardwareSurvey::latest()
                               ->filter($filters)
                               ->paginate($perPage);

        return HardwareSurveyResource::collection($surveys->withQueryString());
    }

    // Note: Default store, show, update, destroy methods from --api are not needed here
    // as we defined specific list methods in the routes.
    // They can be removed or left commented out.

    /**
     * Store a newly created resource in storage.
     */
    // public function store(Request $request)
    // {
    //     //
    // }

    /**
     * Display the specified resource.
     */
    // public function show(string $id)
    // {
    //     //
    // }

    /**
     * Update the specified resource in storage.
     */
    // public function update(Request $request, string $id)
    // {
    //     //
    // }

    /**
     * Remove the specified resource from storage.
     */
    // public function destroy(string $id)
    // {
    //     //
    // }
}
