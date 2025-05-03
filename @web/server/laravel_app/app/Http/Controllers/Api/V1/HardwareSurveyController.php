<?php

namespace App\Http\Controllers\Api\V1;

use App\Http\Controllers\Controller;
use App\Http\Requests\Api\V1\StoreHardwareSurveyRequest;
use App\Models\HardwareSurvey;
use Illuminate\Http\JsonResponse;

class HardwareSurveyController extends Controller
{
    /**
     * Display a listing of the resource.
     */
    public function index()
    {
        //
    }

    /**
     * Store a newly created hardware survey in storage.
     *
     * @param StoreHardwareSurveyRequest $request
     * @return JsonResponse
     */
    public function store(StoreHardwareSurveyRequest $request): JsonResponse
    {
        $validatedData = $request->validated();

        $hardwareSurvey = HardwareSurvey::create($validatedData);

        return response()->json([
            'message' => 'Hardware survey submitted successfully.',
            'data' => $hardwareSurvey
        ], 201);
    }

    /**
     * Display the specified resource.
     */
    public function show(string $id)
    {
        //
    }

    /**
     * Update the specified resource in storage.
     */
    public function update(Request $request, string $id)
    {
        //
    }

    /**
     * Remove the specified resource from storage.
     */
    public function destroy(string $id)
    {
        //
    }
}
