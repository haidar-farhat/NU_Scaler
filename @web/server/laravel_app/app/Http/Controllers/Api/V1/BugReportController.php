<?php

namespace App\Http\Controllers\Api\V1;

use App\Http\Controllers\Controller;
use App\Http\Requests\Api\V1\StoreBugReportRequest;
use App\Models\BugReport;
use Illuminate\Http\JsonResponse;
use App\Http\Responses\ApiResponse;
use App\Services\BugReportService;

class BugReportController extends Controller
{
    protected $bugReportService;

    public function __construct(BugReportService $bugReportService)
    {
        $this->bugReportService = $bugReportService;
    }

    /**
     * Display a listing of the resource.
     */
    public function index()
    {
        //
    }

    /**
     * Store a newly created resource in storage.
     *
     * @param StoreBugReportRequest $request
     * @return JsonResponse
     */
    public function store(StoreBugReportRequest $request): JsonResponse
    {
        $bugReport = $this->bugReportService->create($request->validated());
        return ApiResponse::success('Bug report submitted successfully.', $bugReport, 201);
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
