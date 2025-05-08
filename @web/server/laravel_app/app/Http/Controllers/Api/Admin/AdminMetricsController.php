<?php

namespace App\Http\Controllers\Api\Admin;

use App\Http\Controllers\Controller;
use App\Models\BugReport;
use App\Models\HardwareSurvey;
use App\Models\Review;
use App\Models\User;
use App\Services\AnalyticsService;
use Illuminate\Http\JsonResponse;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Cache;
use Illuminate\Support\Facades\DB;

class AdminMetricsController extends Controller
{
    /**
     * The analytics service instance.
     *
     * @var \App\Services\AnalyticsService
     */
    protected $analyticsService;

    /**
     * Create a new controller instance.
     *
     * @param \App\Services\AnalyticsService $analyticsService
     * @return void
     */
    public function __construct(AnalyticsService $analyticsService)
    {
        $this->analyticsService = $analyticsService;
    }

    /**
     * Get dashboard summary metrics.
     *
     * @return \Illuminate\Http\JsonResponse
     */
    public function dashboard(): JsonResponse
    {
        $metrics = Cache::remember('admin.dashboard.metrics', now()->addHour(), function () {
            return [
                'users' => [
                    'total' => User::count(),
                    'new_today' => User::whereDate('created_at', today())->count(),
                ],
                'reviews' => [
                    'total' => Review::count(),
                    'average_rating' => Review::avg('rating') ?? 0,
                    'new_today' => Review::whereDate('created_at', today())->count(),
                ],
                'bug_reports' => [
                    'total' => BugReport::count(),
                    'new_today' => BugReport::whereDate('created_at', today())->count(),
                    'by_severity' => BugReport::select('severity', DB::raw('count(*) as count'))
                        ->groupBy('severity')
                        ->get()
                        ->pluck('count', 'severity'),
                ],
                'hardware_surveys' => [
                    'total' => HardwareSurvey::count(),
                    'new_today' => HardwareSurvey::whereDate('created_at', today())->count(),
                ],
            ];
        });

        return response()->json(['data' => $metrics]);
    }

    /**
     * Get detailed review metrics.
     *
     * @return \Illuminate\Http\JsonResponse
     */
    public function reviewMetrics(): JsonResponse
    {
        $metrics = $this->analyticsService->getReviewMetrics();
        return response()->json(['data' => $metrics]);
    }

    /**
     * Get detailed bug report metrics.
     *
     * @return \Illuminate\Http\JsonResponse
     */
    public function bugReportMetrics(): JsonResponse
    {
        $metrics = $this->analyticsService->getBugReportMetrics();
        return response()->json(['data' => $metrics]);
    }

    /**
     * Get detailed hardware survey metrics.
     *
     * @return \Illuminate\Http\JsonResponse
     */
    public function hardwareSurveyMetrics(): JsonResponse
    {
        $metrics = $this->analyticsService->getHardwareSurveyMetrics();
        return response()->json(['data' => $metrics]);
    }

    /**
     * Get user growth trends.
     *
     * @param \Illuminate\Http\Request $request
     * @return \Illuminate\Http\JsonResponse
     */
    public function userGrowthTrends(Request $request): JsonResponse
    {
        $period = $request->get('period', 'monthly');

        $trends = Cache::remember("analytics.users.growth.{$period}", now()->addHour(), function () use ($period) {
            $dateFormat = match($period) {
                'daily' => '%Y-%m-%d',
                'weekly' => '%Y-%u',
                'monthly' => '%Y-%m',
                'yearly' => '%Y',
                default => '%Y-%m',
            };

            return User::select(
                DB::raw("DATE_FORMAT(created_at, '{$dateFormat}') as date"),
                DB::raw('count(*) as count')
            )
            ->where('created_at', '>=', now()->subYear())
            ->groupBy('date')
            ->orderBy('date')
            ->get();
        });

        return response()->json(['data' => $trends]);
    }

    /**
     * Get feedback submission trends.
     *
     * @param \Illuminate\Http\Request $request
     * @return \Illuminate\Http\JsonResponse
     */
    public function feedbackTrends(Request $request): JsonResponse
    {
        $period = $request->get('period', 'monthly');
        $type = $request->get('type', 'all');

        $trends = Cache::remember("analytics.feedback.trends.{$type}.{$period}", now()->addHour(), function () use ($period, $type) {
            $dateFormat = match($period) {
                'daily' => '%Y-%m-%d',
                'weekly' => '%Y-%u',
                'monthly' => '%Y-%m',
                'yearly' => '%Y',
                default => '%Y-%m',
            };

            $data = [];

            if ($type == 'all' || $type == 'reviews') {
                $data['reviews'] = Review::select(
                    DB::raw("DATE_FORMAT(created_at, '{$dateFormat}') as date"),
                    DB::raw('count(*) as count')
                )
                ->where('created_at', '>=', now()->subYear())
                ->groupBy('date')
                ->orderBy('date')
                ->get();
            }

            if ($type == 'all' || $type == 'bug_reports') {
                $data['bug_reports'] = BugReport::select(
                    DB::raw("DATE_FORMAT(created_at, '{$dateFormat}') as date"),
                    DB::raw('count(*) as count')
                )
                ->where('created_at', '>=', now()->subYear())
                ->groupBy('date')
                ->orderBy('date')
                ->get();
            }

            if ($type == 'all' || $type == 'hardware_surveys') {
                $data['hardware_surveys'] = HardwareSurvey::select(
                    DB::raw("DATE_FORMAT(created_at, '{$dateFormat}') as date"),
                    DB::raw('count(*) as count')
                )
                ->where('created_at', '>=', now()->subYear())
                ->groupBy('date')
                ->orderBy('date')
                ->get();
            }

            return $data;
        });

        return response()->json(['data' => $trends]);
    }

    /**
     * Get all metrics for export.
     *
     * @return \Illuminate\Http\JsonResponse
     */
    public function exportAllMetrics(): JsonResponse
    {
        $allMetrics = $this->analyticsService->getAllMetrics();

        return response()->json([
            'data' => $allMetrics,
            'generated_at' => now()->toIso8601String(),
            'version' => '1.0',
        ]);
    }
}
