<?php

namespace App\Http\Controllers\Api\V1;

use App\Http\Controllers\Controller;
use App\Http\Requests\Api\V1\StoreReviewRequest; // Use the specific request
use App\Http\Resources\Api\V1\ReviewResource;
use App\Models\Review;
use Illuminate\Http\JsonResponse;
use Illuminate\Http\Request; // Keep for potential future use
use Illuminate\Support\Facades\Auth; // Import Auth facade

class ReviewController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    public function index()
    {
        // To be implemented in Admin API section
        return response()->json(['message' => 'Not implemented'], 501);
    }

    /**
     * Store a newly created resource in storage.
     */
    public function store(StoreReviewRequest $request): JsonResponse
    {
        $validatedData = $request->validated();

        // Add user_uuid if the request is authenticated
        if (Auth::check()) {
            $validatedData['user_uuid'] = Auth::user()->uuid;
        }

        $review = Review::create($validatedData);

        return (new ReviewResource($review))
            ->response()
            ->setStatusCode(201);
    }

    /**
     * Display the specified resource.
     */
    public function show(Review $review)
    {
        // Likely Admin only - implement later
        return response()->json(['message' => 'Not implemented'], 501);
    }

    /**
     * Update the specified resource in storage.
     */
    public function update(Request $request, Review $review)
    {
        // Likely Admin only - implement later
        return response()->json(['message' => 'Not implemented'], 501);
    }

    /**
     * Remove the specified resource from storage.
     */
    public function destroy(Review $review)
    {
        // Likely Admin only - implement later
        return response()->json(['message' => 'Not implemented'], 501);
    }
} 