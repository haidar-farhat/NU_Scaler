<?php

namespace App\Http\Controllers\Api\Admin;

use App\Http\Controllers\Controller;
use Illuminate\Http\JsonResponse;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\File;
use Illuminate\Support\Facades\Storage;

class LogDashboardController extends Controller
{
    /**
     * The available log types.
     *
     * @var array
     */
    protected $logTypes = [
        'api' => 'API Logs',
        'feedback' => 'Feedback Logs',
        'webhooks' => 'Webhook Logs',
        'auth' => 'Authentication Logs',
        'admin' => 'Admin Action Logs',
        'performance' => 'Performance Logs',
        'laravel' => 'System Logs',
    ];

    /**
     * Create a new controller instance.
     *
     * @return void
     */
    public function __construct()
    {
        $this->middleware(['auth:sanctum', 'is_admin']);
    }

    /**
     * Display a list of available log types.
     *
     * @return \Illuminate\Http\JsonResponse
     */
    public function index(): JsonResponse
    {
        return response()->json([
            'data' => $this->logTypes,
        ]);
    }

    /**
     * Display a list of log files for a specific type.
     *
     * @param string $type
     * @return \Illuminate\Http\JsonResponse
     */
    public function listFiles(string $type): JsonResponse
    {
        // Validate log type
        if (!array_key_exists($type, $this->logTypes)) {
            return response()->json([
                'message' => 'Invalid log type',
            ], 400);
        }

        $path = storage_path('logs');
        $files = collect(File::files($path))
            ->filter(function ($file) use ($type) {
                return strpos($file->getFilename(), $type === 'laravel' ? 'laravel' : $type . '-') !== false;
            })
            ->map(function ($file) {
                return [
                    'name' => $file->getFilename(),
                    'size' => $file->getSize(),
                    'modified' => date('Y-m-d H:i:s', $file->getMTime()),
                ];
            })
            ->sortByDesc('modified')
            ->values()
            ->all();

        return response()->json([
            'data' => $files,
        ]);
    }

    /**
     * Display the contents of a log file.
     *
     * @param Request $request
     * @param string $filename
     * @return \Illuminate\Http\JsonResponse
     */
    public function show(Request $request, string $filename): JsonResponse
    {
        $path = storage_path('logs/' . $filename);

        // Check if file exists
        if (!File::exists($path)) {
            return response()->json([
                'message' => 'Log file not found',
            ], 404);
        }

        // Get file size
        $filesize = File::size($path);

        // Default to last 1 MB if file is large
        $maxSize = 1024 * 1024; // 1 MB
        $start = 0;

        if ($filesize > $maxSize) {
            $start = $filesize - $maxSize;
        }

        if ($request->has('start')) {
            $start = (int) $request->get('start');
        }

        $length = min($maxSize, $filesize - $start);

        if ($request->has('length')) {
            $length = min((int) $request->get('length'), $maxSize);
        }

        // Read file
        $handle = fopen($path, 'r');
        fseek($handle, $start);
        $contents = fread($handle, $length);
        fclose($handle);

        return response()->json([
            'data' => [
                'filename' => $filename,
                'filesize' => $filesize,
                'start' => $start,
                'length' => $length,
                'hasMore' => ($start + $length) < $filesize,
                'contents' => base64_encode($contents),
            ],
        ]);
    }

    /**
     * Delete a log file.
     *
     * @param string $filename
     * @return \Illuminate\Http\JsonResponse
     */
    public function destroy(string $filename): JsonResponse
    {
        $path = storage_path('logs/' . $filename);

        // Check if file exists
        if (!File::exists($path)) {
            return response()->json([
                'message' => 'Log file not found',
            ], 404);
        }

        // Delete file
        File::delete($path);

        return response()->json([
            'message' => 'Log file deleted successfully',
        ]);
    }

    /**
     * Get log file statistics.
     *
     * @return \Illuminate\Http\JsonResponse
     */
    public function stats(): JsonResponse
    {
        $path = storage_path('logs');
        $files = File::files($path);

        $stats = [
            'totalFiles' => count($files),
            'totalSize' => array_reduce($files, function ($carry, $file) {
                return $carry + $file->getSize();
            }, 0),
            'oldestFile' => null,
            'newestFile' => null,
            'logsByType' => [],
        ];

        // Get oldest and newest files
        $oldestFile = null;
        $newestFile = null;
        $logsByType = [];

        foreach ($files as $file) {
            // Skip files that don't have a dash (to filter out system files)
            $filename = $file->getFilename();

            // Find log type
            $type = 'other';
            foreach (array_keys($this->logTypes) as $logType) {
                if (strpos($filename, $logType === 'laravel' ? 'laravel' : $logType . '-') !== false) {
                    $type = $logType;
                    break;
                }
            }

            // Count by type
            if (!isset($logsByType[$type])) {
                $logsByType[$type] = [
                    'count' => 0,
                    'size' => 0,
                ];
            }
            $logsByType[$type]['count']++;
            $logsByType[$type]['size'] += $file->getSize();

            // Check for oldest/newest
            $mtime = $file->getMTime();
            if ($oldestFile === null || $mtime < $oldestFile['time']) {
                $oldestFile = [
                    'name' => $filename,
                    'time' => $mtime,
                    'date' => date('Y-m-d H:i:s', $mtime),
                ];
            }
            if ($newestFile === null || $mtime > $newestFile['time']) {
                $newestFile = [
                    'name' => $filename,
                    'time' => $mtime,
                    'date' => date('Y-m-d H:i:s', $mtime),
                ];
            }
        }

        $stats['oldestFile'] = $oldestFile;
        $stats['newestFile'] = $newestFile;
        $stats['logsByType'] = $logsByType;

        return response()->json([
            'data' => $stats,
        ]);
    }

    /**
     * Search logs for a specific term.
     *
     * @param Request $request
     * @return \Illuminate\Http\JsonResponse
     */
    public function search(Request $request): JsonResponse
    {
        $request->validate([
            'term' => 'required|string|min:3',
            'type' => 'nullable|string|in:' . implode(',', array_keys($this->logTypes)),
            'date_from' => 'nullable|date_format:Y-m-d',
            'date_to' => 'nullable|date_format:Y-m-d',
        ]);

        $term = $request->get('term');
        $type = $request->get('type');
        $dateFrom = $request->get('date_from');
        $dateTo = $request->get('date_to');

        $path = storage_path('logs');
        $files = collect(File::files($path));

        // Filter by type
        if ($type) {
            $files = $files->filter(function ($file) use ($type) {
                return strpos($file->getFilename(), $type === 'laravel' ? 'laravel' : $type . '-') !== false;
            });
        }

        // Filter by date range
        if ($dateFrom || $dateTo) {
            $files = $files->filter(function ($file) use ($dateFrom, $dateTo) {
                $fileDate = date('Y-m-d', $file->getMTime());

                if ($dateFrom && $fileDate < $dateFrom) {
                    return false;
                }

                if ($dateTo && $fileDate > $dateTo) {
                    return false;
                }

                return true;
            });
        }

        // Search within files (limit to prevent performance issues)
        $maxFiles = 20;
        $results = [];

        foreach ($files->take($maxFiles) as $file) {
            $filename = $file->getFilename();
            $filePath = $file->getPathname();

            // Use grep for more efficient searching
            $command = sprintf('grep -n "%s" %s', escapeshellarg($term), escapeshellarg($filePath));
            exec($command, $output, $return);

            if (count($output) > 0) {
                // Limit the number of matches per file
                $matches = array_slice($output, 0, 50);

                $results[] = [
                    'filename' => $filename,
                    'matches' => $matches,
                    'match_count' => count($output),
                    'size' => $file->getSize(),
                    'modified' => date('Y-m-d H:i:s', $file->getMTime()),
                ];
            }
        }

        return response()->json([
            'data' => $results,
            'meta' => [
                'total_files_searched' => count($files),
                'max_files_searched' => $maxFiles,
                'total_results' => count($results),
            ],
        ]);
    }
}
