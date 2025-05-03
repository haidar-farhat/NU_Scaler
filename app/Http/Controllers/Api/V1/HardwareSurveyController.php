<?php

namespace App\Http\Controllers\Api\V1;

use App\Http\Controllers\Controller;
use App\Http\Requests\Api\V1\StoreHardwareSurveyRequest; // Use the specific request
use App\Http\Resources\Api\V1\HardwareSurveyResource;
use App\Models\HardwareSurvey;
use Illuminate\Http\JsonResponse;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Auth;

class HardwareSurveyController extends Controller
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
    public function store(StoreHardwareSurveyRequest $request): JsonResponse
    {
        $validatedData = $request->validated();

        // Add user_uuid if the request is authenticated
        if (Auth::check()) {
            $validatedData['user_uuid'] = Auth::user()->uuid;
        }

        $hardwareSurvey = HardwareSurvey::create($validatedData);

        return (new HardwareSurveyResource($hardwareSurvey))
            ->response()
            ->setStatusCode(201);
    }

    /**
     * Display the specified resource.
     */
    public function show(HardwareSurvey $hardwareSurvey)
    {
        // Likely Admin only - implement later
        return response()->json(['message' => 'Not implemented'], 501);
    }

    /**
     * Update the specified resource in storage.
     */
    public function update(Request $request, HardwareSurvey $hardwareSurvey)
    {
        // Likely Admin only - implement later
        return response()->json(['message' => 'Not implemented'], 501);
    }

    /**
     * Remove the specified resource from storage.
     */
    public function destroy(HardwareSurvey $hardwareSurvey)
    {
        // Likely Admin only - implement later
        return response()->json(['message' => 'Not implemented'], 501);
    }
} 