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
            'message' => 'Download link generated successfully.',
            'download_url' => 'https://example.com/downloads/nuscaler-latest',
            'version' => '2.1.0',
            'expires_at' => now()->addDay()->toIso8601String(),
        ];

        return response()->json($downloadInfo);
    }

    /**
     * Get download information for the authenticated user.
     *
     * @param  \Illuminate\Http\Request  $request
     * @return \Illuminate\Http\JsonResponse
     */
    public function getDownloadInfo(Request $request)
    {
        // In a real implementation, we would:
        // 1. Log this download request
        // 2. Generate a signed URL for the S3/storage download
        // 3. Return download details and URL

        // TODO: Implement actual download URL generation
        // For now, return a stub response
        return response()->json([
            'message' => 'Download information retrieved successfully',
            'download' => [
                'version' => '1.0.0',
                'size_mb' => 24.5,
                'release_date' => '2025-05-02',
                'url' => 'https://downloads.nu-scaler.com/releases/nu-scaler-1.0.0.zip',
                'expires_at' => now()->addHours(1)->toIso8601String(),
            ],
        ]);
    }

    // We might add methods later to list download history for a user or admin
}
