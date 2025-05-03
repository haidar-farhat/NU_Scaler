<?php

namespace App\Http\Controllers\Api\V1;

use App\Http\Controllers\Controller;
use App\Http\Requests\Api\V1\StoreBugReportRequest; // Use the specific request
use App\Http\Resources\Api\V1\BugReportResource;
use App\Models\BugReport;
use Illuminate\Http\JsonResponse;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Auth;

class BugReportController extends Controller
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
    public function store(StoreBugReportRequest $request): JsonResponse
    {
        $validatedData = $request->validated();

        // Add user_uuid if the request is authenticated
        if (Auth::check()) {
            $validatedData['user_uuid'] = Auth::user()->uuid;
        }

        // TODO: Handle potential log file upload if needed

        $bugReport = BugReport::create($validatedData);

        return (new BugReportResource($bugReport))
            ->response()
            ->setStatusCode(201);
    }

    /**
     * Display the specified resource.
     */
    public function show(BugReport $bugReport)
    {
        // Likely Admin only - implement later
        return response()->json(['message' => 'Not implemented'], 501);
    }

    /**
     * Update the specified resource in storage.
     */
    public function update(Request $request, BugReport $bugReport)
    {
        // Likely Admin only - implement later
        return response()->json(['message' => 'Not implemented'], 501);
    }

    /**
     * Remove the specified resource from storage.
     */
    public function destroy(BugReport $bugReport)
    {
        // Likely Admin only - implement later
        return response()->json(['message' => 'Not implemented'], 501);
    }
} 