<?php

namespace App\Http\Controllers\Api\Admin;

use App\Http\Controllers\Controller;
use App\Models\BugReport;
use App\Models\HardwareSurvey;
use App\Models\Review;
use Illuminate\Http\Request;
use Illuminate\Http\JsonResponse;

class FeedbackController extends Controller
{
    /**
     * Display a listing of the reviews.
     */
    public function listReviews(Request $request): JsonResponse
    {
        // Add pagination later if needed
        $reviews = Review::latest()->get();
        return response()->json($reviews);
    }

    /**
     * Display a listing of the bug reports.
     */
    public function listBugReports(Request $request): JsonResponse
    {
        $bugReports = BugReport::latest()->get();
        return response()->json($bugReports);
    }

    /**
     * Display a listing of the hardware surveys.
     */
    public function listHardwareSurveys(Request $request): JsonResponse
    {
        $surveys = HardwareSurvey::latest()->get();
        return response()->json($surveys);
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
