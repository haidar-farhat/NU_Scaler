<?php

namespace App\Http\Controllers\Api\V1;

use App\Http\Controllers\Controller;
use Illuminate\Http\Request;
use App\Models\DownloadLog;
use Illuminate\Http\JsonResponse;
use Illuminate\Support\Facades\Log;

class DownloadController extends Controller
{
    /**
     * Handle a request for the application download link.
     *
     * Logs the attempt and returns a (placeholder) link or file info.
     *
     * @param Request $request
     * @return JsonResponse
     */
    public function getDownloadLink(Request $request): JsonResponse
    {
        $user = $request->user();

        try {
            DownloadLog::create([
                'user_id' => $user->id,
                'ip_address' => $request->ip()
            ]);
        } catch (\Exception $e) {
            Log::error('Failed to create download log', [
                'user_id' => $user->id,
                'ip_address' => $request->ip(),
                'error' => $e->getMessage()
            ]);
        }

        $downloadInfo = [
            'message' => 'Download initiated successfully.',
            'installer_url' => 'https://example.com/downloads/nuscaler-latest.exe',
            'version' => '1.0.0',
        ];

        return response()->json($downloadInfo);
    }

    // We might add methods later to list download history for a user or admin
}
