<?php

namespace App\Http\Controllers\Api\Admin;

use App\Http\Controllers\Controller;
use App\Models\BugReport;
use App\Models\HardwareSurvey;
use App\Models\Review;
use Illuminate\Http\JsonResponse;
use Illuminate\Http\Request;
use Illuminate\Support\Carbon;
use Illuminate\Support\Facades\DB;

class MetricsController extends Controller
{
    /**
     * Get rating distribution for reviews.
     *
     * @param Request $request
     * @return JsonResponse
     */
    public function reviewsDistribution(Request $request): JsonResponse
    {
        $timeframe = $request->get('timeframe', 'all');
        $query = Review::query();
        
        // Apply date filters
        $query = $this->applyTimeframeFilter($query, $timeframe);
        
        // Get counts by rating
        $distribution = $query->select('rating', DB::raw('count(*) as count'))
            ->groupBy('rating')
            ->orderBy('rating')
            ->get()
            ->pluck('count', 'rating')
            ->toArray();
        
        // Ensure all rating values (1-5) exist in the response
        $completeDistribution = [];
        for ($i = 1; $i <= 5; $i++) {
            $completeDistribution[$i] = $distribution[$i] ?? 0;
        }
        
        // Calculate the average rating
        $averageRating = $query->avg('rating') ?? 0;
        
        return response()->json([
            'distribution' => $completeDistribution,
            'average_rating' => round($averageRating, 2),
            'total_reviews' => array_sum($completeDistribution),
            'timeframe' => $timeframe,
        ]);
    }
    
    /**
     * Get bug report distribution by severity.
     *
     * @param Request $request
     * @return JsonResponse
     */
    public function bugReportsSeverity(Request $request): JsonResponse
    {
        $timeframe = $request->get('timeframe', 'all');
        $query = BugReport::query();
        
        // Apply date filters
        $query = $this->applyTimeframeFilter($query, $timeframe);
        
        // Get counts by severity
        $distribution = $query->select('severity', DB::raw('count(*) as count'))
            ->groupBy('severity')
            ->orderBy(DB::raw('FIELD(severity, "low", "medium", "high", "critical")'))
            ->get()
            ->pluck('count', 'severity')
            ->toArray();
        
        // Ensure all severity values exist in the response
        $severityLevels = ['low', 'medium', 'high', 'critical'];
        $completeDistribution = [];
        foreach ($severityLevels as $level) {
            $completeDistribution[$level] = $distribution[$level] ?? 0;
        }
        
        return response()->json([
            'distribution' => $completeDistribution,
            'total_bug_reports' => array_sum($completeDistribution),
            'timeframe' => $timeframe,
        ]);
    }
    
    /**
     * Get hardware survey distribution by operating system.
     *
     * @param Request $request
     * @return JsonResponse
     */
    public function hardwareOsDistribution(Request $request): JsonResponse
    {
        $timeframe = $request->get('timeframe', 'all');
        $query = HardwareSurvey::query();
        
        // Apply date filters
        $query = $this->applyTimeframeFilter($query, $timeframe);
        
        // Get counts by OS
        $distribution = $query->select('os', DB::raw('count(*) as count'))
            ->groupBy('os')
            ->orderBy('count', 'desc')
            ->get()
            ->pluck('count', 'os')
            ->toArray();
        
        return response()->json([
            'distribution' => $distribution,
            'total_surveys' => array_sum($distribution),
            'timeframe' => $timeframe,
        ]);
    }
    
    /**
     * Get hardware survey distribution by GPU.
     *
     * @param Request $request
     * @return JsonResponse
     */
    public function hardwareGpuDistribution(Request $request): JsonResponse
    {
        $timeframe = $request->get('timeframe', 'all');
        $query = HardwareSurvey::query();
        
        // Apply date filters
        $query = $this->applyTimeframeFilter($query, $timeframe);
        
        // Get counts by GPU (grouped by manufacturer for cleaner data)
        $gpuData = $query->select('gpu', DB::raw('count(*) as count'))
            ->groupBy('gpu')
            ->orderBy('count', 'desc')
            ->limit(10) // Limit to top 10 GPUs
            ->get();
        
        // Group by manufacturer
        $manufacturerMapping = [
            'nvidia' => ['nvidia', 'geforce', 'rtx', 'gtx'],
            'amd' => ['amd', 'radeon', 'rx'],
            'intel' => ['intel', 'arc', 'iris'],
        ];
        
        $manufacturerDistribution = [];
        $gpuDistribution = [];
        
        foreach ($gpuData as $item) {
            $gpuDistribution[$item->gpu] = $item->count;
            
            $gpu = strtolower($item->gpu);
            $assigned = false;
            
            foreach ($manufacturerMapping as $manufacturer => $keywords) {
                foreach ($keywords as $keyword) {
                    if (strpos($gpu, $keyword) !== false) {
                        $manufacturerDistribution[$manufacturer] = 
                            ($manufacturerDistribution[$manufacturer] ?? 0) + $item->count;
                        $assigned = true;
                        break;
                    }
                }
                
                if ($assigned) {
                    break;
                }
            }
            
            if (!$assigned) {
                $manufacturerDistribution['other'] = 
                    ($manufacturerDistribution['other'] ?? 0) + $item->count;
            }
        }
        
        return response()->json([
            'manufacturer_distribution' => $manufacturerDistribution,
            'detailed_distribution' => $gpuDistribution,
            'total_surveys' => array_sum($gpuDistribution),
            'timeframe' => $timeframe,
        ]);
    }
    
    /**
     * Get feedback submission trends over time.
     *
     * @param Request $request
     * @return JsonResponse
     */
    public function submissionTrends(Request $request): JsonResponse
    {
        $timeframe = $request->get('timeframe', 'month');
        $period = $request->get('period', 'day');
        
        // Determine date format and grouping based on period
        switch ($period) {
            case 'hour':
                $format = 'Y-m-d H:00';
                $rawFormat = "DATE_FORMAT(created_at, '%Y-%m-%d %H:00')";
                break;
            case 'day':
                $format = 'Y-m-d';
                $rawFormat = "DATE(created_at)";
                break;
            case 'week':
                $format = 'Y-W';
                $rawFormat = "CONCAT(YEAR(created_at), '-', WEEK(created_at))";
                break;
            case 'month':
                $format = 'Y-m';
                $rawFormat = "DATE_FORMAT(created_at, '%Y-%m')";
                break;
            default:
                $format = 'Y-m-d';
                $rawFormat = "DATE(created_at)";
        }
        
        // Set date range based on timeframe
        $startDate = $this->getStartDateForTimeframe($timeframe);
        
        // Get data for each feedback type
        $reviewTrends = $this->getTrendData(Review::class, $startDate, $rawFormat, $format);
        $bugReportTrends = $this->getTrendData(BugReport::class, $startDate, $rawFormat, $format);
        $hardwareSurveyTrends = $this->getTrendData(HardwareSurvey::class, $startDate, $rawFormat, $format);
        
        // Merge the data points
        $allTrends = $this->mergeTrendData(
            $reviewTrends, 
            $bugReportTrends, 
            $hardwareSurveyTrends,
            $startDate,
            $format,
            $period
        );
        
        return response()->json([
            'trends' => $allTrends,
            'timeframe' => $timeframe,
            'period' => $period,
        ]);
    }
    
    /**
     * Apply timeframe filter to a query.
     *
     * @param \Illuminate\Database\Eloquent\Builder $query
     * @param string $timeframe
     * @return \Illuminate\Database\Eloquent\Builder
     */
    private function applyTimeframeFilter($query, string $timeframe)
    {
        $startDate = $this->getStartDateForTimeframe($timeframe);
        
        if ($startDate) {
            return $query->where('created_at', '>=', $startDate);
        }
        
        return $query;
    }
    
    /**
     * Get the start date for a given timeframe.
     *
     * @param string $timeframe
     * @return Carbon|null
     */
    private function getStartDateForTimeframe(string $timeframe): ?Carbon
    {
        switch ($timeframe) {
            case 'day':
                return Carbon::now()->subDay();
            case 'week':
                return Carbon::now()->subWeek();
            case 'month':
                return Carbon::now()->subMonth();
            case 'quarter':
                return Carbon::now()->subQuarter();
            case 'year':
                return Carbon::now()->subYear();
            case 'all':
                return null;
            default:
                return null;
        }
    }
    
    /**
     * Get trend data for a specific model.
     *
     * @param string $modelClass
     * @param Carbon|null $startDate
     * @param string $rawFormat
     * @param string $format
     * @return array
     */
    private function getTrendData(string $modelClass, ?Carbon $startDate, string $rawFormat, string $format): array
    {
        $query = $modelClass::query();
        
        if ($startDate) {
            $query->where('created_at', '>=', $startDate);
        }
        
        $data = $query->select(DB::raw("$rawFormat as date"), DB::raw('count(*) as count'))
            ->groupBy('date')
            ->orderBy('date')
            ->get()
            ->pluck('count', 'date')
            ->toArray();
            
        return $data;
    }
    
    /**
     * Merge trend data from multiple models into one dataset.
     *
     * @param array $reviewTrends
     * @param array $bugReportTrends
     * @param array $hardwareSurveyTrends
     * @param Carbon|null $startDate
     * @param string $format
     * @param string $period
     * @return array
     */
    private function mergeTrendData(
        array $reviewTrends, 
        array $bugReportTrends, 
        array $hardwareSurveyTrends,
        ?Carbon $startDate,
        string $format,
        string $period
    ): array {
        $result = [];
        
        // Generate all date points within the range
        if ($startDate) {
            $currentDate = clone $startDate;
            $endDate = Carbon::now();
            
            while ($currentDate <= $endDate) {
                $formattedDate = $currentDate->format($format);
                
                $result[$formattedDate] = [
                    'date' => $formattedDate,
                    'reviews' => 0,
                    'bug_reports' => 0,
                    'hardware_surveys' => 0,
                    'total' => 0
                ];
                
                // Increment by period
                switch ($period) {
                    case 'hour':
                        $currentDate->addHour();
                        break;
                    case 'day':
                        $currentDate->addDay();
                        break;
                    case 'week':
                        $currentDate->addWeek();
                        break;
                    case 'month':
                        $currentDate->addMonth();
                        break;
                    default:
                        $currentDate->addDay();
                }
            }
        }
        
        // Fill in actual data points
        foreach ($reviewTrends as $date => $count) {
            if (isset($result[$date])) {
                $result[$date]['reviews'] = $count;
                $result[$date]['total'] += $count;
            } else {
                $result[$date] = [
                    'date' => $date,
                    'reviews' => $count,
                    'bug_reports' => 0,
                    'hardware_surveys' => 0,
                    'total' => $count
                ];
            }
        }
        
        foreach ($bugReportTrends as $date => $count) {
            if (isset($result[$date])) {
                $result[$date]['bug_reports'] = $count;
                $result[$date]['total'] += $count;
            } else {
                $result[$date] = [
                    'date' => $date,
                    'reviews' => 0,
                    'bug_reports' => $count,
                    'hardware_surveys' => 0,
                    'total' => $count
                ];
            }
        }
        
        foreach ($hardwareSurveyTrends as $date => $count) {
            if (isset($result[$date])) {
                $result[$date]['hardware_surveys'] = $count;
                $result[$date]['total'] += $count;
            } else {
                $result[$date] = [
                    'date' => $date,
                    'reviews' => 0,
                    'bug_reports' => 0,
                    'hardware_surveys' => $count,
                    'total' => $count
                ];
            }
        }
        
        // Convert to indexed array and sort by date
        $result = array_values($result);
        usort($result, function ($a, $b) {
            return strcmp($a['date'], $b['date']);
        });
        
        return $result;
    }
}