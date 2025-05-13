<?php

namespace App\Http\Controllers\Api\V1;

use App\Http\Controllers\Controller;
use Illuminate\Http\Request;
use App\Models\DownloadLog;
use Illuminate\Http\JsonResponse;
use Illuminate\Support\Facades\Log;
use Illuminate\Support\Facades\File;
use Illuminate\Support\Facades\Storage;
use Symfony\Component\HttpFoundation\BinaryFileResponse;

class DownloadController extends Controller
{
    /**
     * Handle a request for the application download link.
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

        // Generate a signed download URL
        $exePath = base_path('../../releases/NuScaler.exe');

        if (!File::exists($exePath)) {
            Log::error('Download file not found', [
                'path' => $exePath,
                'user_id' => $user->id
            ]);

            return response()->json([
                'message' => 'Download file not available.',
                'error' => 'File not found'
            ], 404);
        }

        // Create a signed URL for the download
        $downloadUrl = route('api.v1.download.file', [
            'platform' => 'windows',
            'token' => encrypt($user->id . '_' . now()->timestamp)
        ]);

        $downloadInfo = [
            'message' => 'Download link generated successfully.',
            'download_url' => $downloadUrl,
            'version' => '2.1.0',
            'size_mb' => round(File::size($exePath) / (1024 * 1024), 2),
            'expires_at' => now()->addDay()->toIso8601String(),
        ];

        return response()->json($downloadInfo);
    }

    /**
     * Serve the actual executable file for download
     *
     * @param Request $request
     * @param string $platform
     * @return BinaryFileResponse|JsonResponse
     */
    public function downloadFile(Request $request, string $platform)
    {
        try {
            // Validate the token
            $token = $request->query('token');
            if (!$token) {
                return response()->json(['message' => 'Invalid download token'], 401);
            }

            // Allow public token for testing purposes
            $isPublicAccess = ($token === 'public-access');

            // Only decrypt non-public tokens
            if (!$isPublicAccess) {
                try {
                    // Try to decrypt the token - this will fail if token is invalid
                    $decryptedToken = decrypt($token);
                } catch (\Exception $e) {
                    return response()->json(['message' => 'Invalid or expired token'], 401);
                }
            }

            // The path to the exe file
            $exePath = base_path('../../releases/NuScaler.exe');

            if (!File::exists($exePath)) {
                Log::error('Download file not found during download attempt', [
                    'path' => $exePath
                ]);
                return response()->json(['message' => 'File not found'], 404);
            }

            // Log the actual download
            if ($request->user()) {
                try {
                    DownloadLog::create([
                        'user_id' => $request->user()->id,
                        'ip_address' => $request->ip(),
                        'platform' => $platform,
                        'downloaded' => true
                    ]);
                } catch (\Exception $e) {
                    Log::error('Failed to log file download', ['error' => $e->getMessage()]);
                }
            } else if ($isPublicAccess) {
                // Log anonymous download
                Log::info('Anonymous download via public link', [
                    'ip' => $request->ip(),
                    'platform' => $platform
                ]);
            }

            // Return the file as a download
            return response()->download($exePath, 'NuScaler.exe', [
                'Content-Type' => 'application/octet-stream',
                'Content-Disposition' => 'attachment; filename="NuScaler.exe"'
            ]);
        } catch (\Exception $e) {
            Log::error('Download error', [
                'error' => $e->getMessage(),
                'trace' => $e->getTraceAsString()
            ]);

            return response()->json([
                'message' => 'Failed to process download',
                'error' => $e->getMessage()
            ], 500);
        }
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

    /**
     * Get download link without authentication (for testing)
     *
     * @return JsonResponse
     */
    public function getPublicDownloadLink(): JsonResponse
    {
        try {
            // The path to the exe file
            $exePath = base_path('../../releases/NuScaler.exe');

            if (!File::exists($exePath)) {
                Log::error('Download file not found in public link', [
                    'path' => $exePath
                ]);

                return response()->json([
                    'message' => 'Download file not available.',
                    'error' => 'File not found'
                ], 404);
            }

            // Create a direct download link without encryption
            $downloadUrl = route('api.v1.download.file', [
                'platform' => 'windows',
                'token' => 'public-access'
            ]);

            $downloadInfo = [
                'message' => 'Public download link generated successfully.',
                'download_url' => $downloadUrl,
                'version' => '2.1.0',
                'size_mb' => round(File::size($exePath) / (1024 * 1024), 2),
                'expires_at' => now()->addDay()->toIso8601String(),
                'is_public' => true
            ];

            return response()->json($downloadInfo);
        } catch (\Exception $e) {
            Log::error('Failed to create public download link', [
                'error' => $e->getMessage(),
                'trace' => $e->getTraceAsString()
            ]);

            return response()->json([
                'message' => 'Failed to generate download link',
                'error' => $e->getMessage()
            ], 500);
        }
    }

    // We might add methods later to list download history for a user or admin
}
