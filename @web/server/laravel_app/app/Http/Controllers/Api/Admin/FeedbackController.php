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
        $perPage = $request->query('per_page', 15); // Default 15 items per page
        $reviews = Review::latest()->paginate($perPage);
        return ReviewResource::collection($reviews);
    }

    /**
     * Display a paginated listing of the bug reports.
     */
    public function listBugReports(Request $request): AnonymousResourceCollection
    {
        $perPage = $request->query('per_page', 15);
        $bugReports = BugReport::latest()->paginate($perPage);
        return BugReportResource::collection($bugReports);
    }

    /**
     * Display a paginated listing of the hardware surveys.
     */
    public function listHardwareSurveys(Request $request): AnonymousResourceCollection
    {
        $perPage = $request->query('per_page', 15);
        $surveys = HardwareSurvey::latest()->paginate($perPage);
        return HardwareSurveyResource::collection($surveys);
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
