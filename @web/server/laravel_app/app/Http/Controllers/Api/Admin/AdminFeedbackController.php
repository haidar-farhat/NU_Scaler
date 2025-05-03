<?php

namespace App\Http\Controllers\Api\Admin;

use App\Http\Controllers\Controller;
use Illuminate\Http\Request;
use App\Models\Review;

class AdminFeedbackController extends Controller
{
    /**
     * Display a listing of feedback items.
     *
     * @param  \Illuminate\Http\Request  $request
     * @return \Illuminate\Http\JsonResponse
     */
    public function index(Request $request)
    {
        // Get reviews with optional filters
        $reviews = Review::query()
            ->filter($request->only(['rating']))
            ->latest()
            ->paginate(15);

        // Laravel's paginate() will automatically include the pagination metadata
        // when converted to JSON, but we need to ensure we don't wrap it in another
        // array that would change the expected structure
        return $reviews;
    }

    /**
     * Display the specified feedback item.
     *
     * @param  int  $id
     * @return \Illuminate\Http\JsonResponse
     */
    public function show($id)
    {
        // TODO: Implement feedback detail view
        return response()->json([
            'message' => 'Feedback detail endpoint',
            'id' => $id,
            'data' => null,
        ]);
    }

    /**
     * Update the specified feedback item.
     *
     * @param  \Illuminate\Http\Request  $request
     * @param  int  $id
     * @return \Illuminate\Http\JsonResponse
     */
    public function update(Request $request, $id)
    {
        // TODO: Implement feedback update
        return response()->json([
            'message' => 'Feedback updated successfully',
            'id' => $id,
        ]);
    }

    /**
     * Remove the specified feedback item.
     *
     * @param  int  $id
     * @return \Illuminate\Http\JsonResponse
     */
    public function destroy($id)
    {
        // TODO: Implement feedback deletion
        return response()->json([
            'message' => 'Feedback deleted successfully',
            'id' => $id,
        ]);
    }
}
