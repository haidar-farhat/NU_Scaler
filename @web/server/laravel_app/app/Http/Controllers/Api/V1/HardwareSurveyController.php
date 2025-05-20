<?php

namespace App\Http\Controllers\Api\V1;

use App\Http\Controllers\Controller;
use App\Http\Requests\Api\V1\StoreHardwareSurveyRequest;
use App\Models\HardwareSurvey;
use Illuminate\Http\JsonResponse;
use App\Http\Responses\ApiResponse;
use App\Services\HardwareSurveyService;

class HardwareSurveyController extends Controller
{
    protected $hardwareSurveyService;

    public function __construct(HardwareSurveyService $hardwareSurveyService)
    {
        $this->hardwareSurveyService = $hardwareSurveyService;
    }

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
        $hardwareSurvey = $this->hardwareSurveyService->create($request->validated());
        return ApiResponse::success('Hardware survey submitted successfully.', $hardwareSurvey, 201);
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
