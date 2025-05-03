<?php

namespace App\Http\Controllers\Api\V1;

use App\Http\Controllers\Controller;
use App\Http\Requests\Api\V1\StoreBugReportRequest;
use App\Models\BugReport;
use Illuminate\Http\JsonResponse;

class BugReportController extends Controller
{
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
        $validatedData = $request->validated();

        $bugReport = BugReport::create($validatedData);

        return response()->json([
            'message' => 'Bug report submitted successfully.',
            'data' => $bugReport
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
