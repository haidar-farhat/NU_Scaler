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
     * Get metrics for active users who have submitted multiple types of feedback.
     *
     * @param Request $request
     * @return JsonResponse
     */
    public function activeFeedbackUsers(Request $request): JsonResponse
    {
        $timeframe = $request->get('timeframe', 'month');
        $minSubmissions = max(1, min(10, $request->get('min_submissions', 2)));
        
        // Set date range based on timeframe
        $startDate = $this->getStartDateForTimeframe($timeframe);
        
        // Base query for users with reviews
        $reviewUsers = Review::query()
            ->when($startDate, function ($query) use ($startDate) {
                $query->where('created_at', '>=', $startDate);
            })
            ->whereNotNull('user_uuid')
            ->select('user_uuid', DB::raw('count(*) as review_count'))
            ->groupBy('user_uuid');
            
        // Base query for users with bug reports
        $bugReportUsers = BugReport::query()
            ->when($startDate, function ($query) use ($startDate) {
                $query->where('created_at', '>=', $startDate);
            })
            ->whereNotNull('user_uuid')
            ->select('user_uuid', DB::raw('count(*) as bug_report_count'))
            ->groupBy('user_uuid');
            
        // Base query for users with hardware surveys
        $hardwareSurveyUsers = HardwareSurvey::query()
            ->when($startDate, function ($query) use ($startDate) {
                $query->where('created_at', '>=', $startDate);
            })
            ->whereNotNull('user_uuid')
            ->select('user_uuid', DB::raw('count(*) as hardware_survey_count'))
            ->groupBy('user_uuid');
            
        // Combine the results using a query builder to get all user activity
        $userActivity = DB::table(DB::raw("({$reviewUsers->toSql()}) as reviews"))
            ->mergeBindings($reviewUsers->getQuery())
            ->select(
                'reviews.user_uuid',
                'reviews.review_count',
                DB::raw('COALESCE(bug_reports.bug_report_count, 0) as bug_report_count'),
                DB::raw('COALESCE(hardware_surveys.hardware_survey_count, 0) as hardware_survey_count'),
                DB::raw('(reviews.review_count + COALESCE(bug_reports.bug_report_count, 0) + COALESCE(hardware_surveys.hardware_survey_count, 0)) as total_submissions')
            )
            ->leftJoin(
                DB::raw("({$bugReportUsers->toSql()}) as bug_reports"),
                'reviews.user_uuid', '=', 'bug_reports.user_uuid'
            )
            ->mergeBindings($bugReportUsers->getQuery())
            ->leftJoin(
                DB::raw("({$hardwareSurveyUsers->toSql()}) as hardware_surveys"),
                'reviews.user_uuid', '=', 'hardware_surveys.user_uuid'
            )
            ->mergeBindings($hardwareSurveyUsers->getQuery())
            ->having('total_submissions', '>=', $minSubmissions)
            ->orderBy('total_submissions', 'desc')
            ->get();
            
        // Get metrics breakdown by submission counts
        $submissionCounts = [];
        for ($i = $minSubmissions; $i <= 10; $i++) {
            $submissionCounts[$i] = $userActivity->filter(function ($user) use ($i) {
                return $user->total_submissions == $i;
            })->count();
        }
        
        // Calculate feedback type distribution
        $feedbackDistribution = [
            'only_reviews' => $userActivity->filter(function ($user) {
                return $user->review_count > 0 && $user->bug_report_count == 0 && $user->hardware_survey_count == 0;
            })->count(),
            'only_bug_reports' => $userActivity->filter(function ($user) {
                return $user->review_count == 0 && $user->bug_report_count > 0 && $user->hardware_survey_count == 0;
            })->count(),
            'only_hardware_surveys' => $userActivity->filter(function ($user) {
                return $user->review_count == 0 && $user->bug_report_count == 0 && $user->hardware_survey_count > 0;
            })->count(),
            'multiple_types' => $userActivity->filter(function ($user) {
                $typesSubmitted = 0;
                if ($user->review_count > 0) $typesSubmitted++;
                if ($user->bug_report_count > 0) $typesSubmitted++;
                if ($user->hardware_survey_count > 0) $typesSubmitted++;
                return $typesSubmitted > 1;
            })->count(),
        ];
        
        return response()->json([
            'total_active_users' => $userActivity->count(),
            'submission_counts' => $submissionCounts,
            'feedback_distribution' => $feedbackDistribution,
            'most_active_users' => $userActivity->take(10),
            'timeframe' => $timeframe,
        ]);
    }
    
    /**
     * Get correlation metrics between different types of feedback.
     *
     * @param Request $request
     * @return JsonResponse
     */
    public function feedbackCorrelation(Request $request): JsonResponse
    {
        $timeframe = $request->get('timeframe', 'month');
        
        // Set date range based on timeframe
        $startDate = $this->getStartDateForTimeframe($timeframe);
        
        // Get reviews with a rating breakdown
        $reviewStats = Review::query()
            ->when($startDate, function ($query) use ($startDate) {
                $query->where('created_at', '>=', $startDate);
            })
            ->select('rating', DB::raw('count(*) as count'))
            ->groupBy('rating')
            ->orderBy('rating')
            ->get()
            ->pluck('count', 'rating')
            ->toArray();
            
        // Get bug reports with a severity breakdown
        $bugReportStats = BugReport::query()
            ->when($startDate, function ($query) use ($startDate) {
                $query->where('created_at', '>=', $startDate);
            })
            ->select('severity', DB::raw('count(*) as count'))
            ->groupBy('severity')
            ->orderBy(DB::raw('FIELD(severity, "low", "medium", "high", "critical")'))
            ->get()
            ->pluck('count', 'severity')
            ->toArray();
            
        // Get hardware survey OS distribution
        $hardwareOsStats = HardwareSurvey::query()
            ->when($startDate, function ($query) use ($startDate) {
                $query->where('created_at', '>=', $startDate);
            })
            ->select('os', DB::raw('count(*) as count'))
            ->groupBy('os')
            ->orderBy('count', 'desc')
            ->limit(5)
            ->get()
            ->pluck('count', 'os')
            ->toArray();
            
        // Analyze correlation between ratings and bug report severity
        $ratingBugSeverityCorrelation = $this->analyzeRatingBugSeverityCorrelation($startDate);
        
        // Analyze correlation between OS and ratings
        $osRatingCorrelation = $this->analyzeOsRatingCorrelation($startDate);
        
        // Analyze users who submit both positive reviews and bug reports
        $userFeedbackPatterns = $this->analyzeUserFeedbackPatterns($startDate);
        
        return response()->json([
            'review_stats' => $reviewStats,
            'bug_report_stats' => $bugReportStats,
            'hardware_os_stats' => $hardwareOsStats,
            'rating_bug_severity_correlation' => $ratingBugSeverityCorrelation,
            'os_rating_correlation' => $osRatingCorrelation,
            'user_feedback_patterns' => $userFeedbackPatterns,
            'timeframe' => $timeframe,
        ]);
    }
    
    /**
     * Analyze correlation between ratings and bug report severity.
     *
     * @param Carbon|null $startDate
     * @return array
     */
    private function analyzeRatingBugSeverityCorrelation(?Carbon $startDate): array
    {
        // For users who submitted both reviews and bug reports, check for correlations
        $usersWithBoth = DB::table('reviews as r')
            ->select('r.user_uuid', 'r.rating', 'b.severity')
            ->join('bug_reports as b', 'r.user_uuid', '=', 'b.user_uuid')
            ->whereNotNull('r.user_uuid')
            ->when($startDate, function ($query) use ($startDate) {
                $query->where('r.created_at', '>=', $startDate)
                      ->where('b.created_at', '>=', $startDate);
            })
            ->get();
            
        // Initialize correlation matrix
        $correlationMatrix = [];
        for ($rating = 1; $rating <= 5; $rating++) {
            $correlationMatrix[$rating] = [
                'low' => 0,
                'medium' => 0,
                'high' => 0,
                'critical' => 0,
            ];
        }
        
        // Fill correlation matrix
        foreach ($usersWithBoth as $userFeedback) {
            $correlationMatrix[$userFeedback->rating][$userFeedback->severity]++;
        }
        
        // Calculate percentages for easier interpretation
        $percentageMatrix = [];
        foreach ($correlationMatrix as $rating => $severities) {
            $total = array_sum($severities);
            if ($total > 0) {
                $percentageMatrix[$rating] = [];
                foreach ($severities as $severity => $count) {
                    $percentageMatrix[$rating][$severity] = round(($count / $total) * 100, 1);
                }
            }
        }
        
        return [
            'count_matrix' => $correlationMatrix,
            'percentage_matrix' => $percentageMatrix,
            'total_users_with_both' => count($usersWithBoth),
        ];
    }
    
    /**
     * Analyze correlation between OS and ratings.
     *
     * @param Carbon|null $startDate
     * @return array
     */
    private function analyzeOsRatingCorrelation(?Carbon $startDate): array
    {
        // For users who submitted both hardware surveys and reviews
        $usersWithBoth = DB::table('hardware_surveys as h')
            ->select('h.user_uuid', 'h.os', 'r.rating')
            ->join('reviews as r', 'h.user_uuid', '=', 'r.user_uuid')
            ->whereNotNull('h.user_uuid')
            ->when($startDate, function ($query) use ($startDate) {
                $query->where('h.created_at', '>=', $startDate)
                      ->where('r.created_at', '>=', $startDate);
            })
            ->get();
            
        // Group by OS to get average ratings
        $osByRating = [];
        $osUserCounts = [];
        
        foreach ($usersWithBoth as $userFeedback) {
            $os = $userFeedback->os;
            $rating = $userFeedback->rating;
            
            if (!isset($osByRating[$os])) {
                $osByRating[$os] = [];
                $osUserCounts[$os] = 0;
            }
            
            $osByRating[$os][] = $rating;
            $osUserCounts[$os]++;
        }
        
        // Calculate average rating for each OS
        $averageRatingByOs = [];
        foreach ($osByRating as $os => $ratings) {
            $averageRatingByOs[$os] = [
                'average_rating' => round(array_sum($ratings) / count($ratings), 2),
                'user_count' => $osUserCounts[$os],
            ];
        }
        
        // Sort by highest average rating
        arsort($averageRatingByOs);
        
        return [
            'average_rating_by_os' => $averageRatingByOs,
            'total_users_with_both' => count($usersWithBoth),
        ];
    }
    
    /**
     * Analyze user feedback patterns.
     *
     * @param Carbon|null $startDate
     * @return array
     */
    private function analyzeUserFeedbackPatterns(?Carbon $startDate): array
    {
        // Get users who submitted reviews
        $reviewUsers = Review::query()
            ->when($startDate, function ($query) use ($startDate) {
                $query->where('created_at', '>=', $startDate);
            })
            ->whereNotNull('user_uuid')
            ->select('user_uuid')
            ->distinct()
            ->get()
            ->pluck('user_uuid')
            ->toArray();
            
        // Get users who submitted bug reports
        $bugReportUsers = BugReport::query()
            ->when($startDate, function ($query) use ($startDate) {
                $query->where('created_at', '>=', $startDate);
            })
            ->whereNotNull('user_uuid')
            ->select('user_uuid')
            ->distinct()
            ->get()
            ->pluck('user_uuid')
            ->toArray();
            
        // Get users who submitted hardware surveys
        $hardwareSurveyUsers = HardwareSurvey::query()
            ->when($startDate, function ($query) use ($startDate) {
                $query->where('created_at', '>=', $startDate);
            })
            ->whereNotNull('user_uuid')
            ->select('user_uuid')
            ->distinct()
            ->get()
            ->pluck('user_uuid')
            ->toArray();
            
        // Calculate intersections
        $reviewAndBugReports = array_intersect($reviewUsers, $bugReportUsers);
        $reviewAndHardwareSurveys = array_intersect($reviewUsers, $hardwareSurveyUsers);
        $bugReportsAndHardwareSurveys = array_intersect($bugReportUsers, $hardwareSurveyUsers);
        $allThree = array_intersect($reviewUsers, $bugReportUsers, $hardwareSurveyUsers);
        
        // Calculate unique submissions
        $onlyReviews = array_diff($reviewUsers, $bugReportUsers, $hardwareSurveyUsers);
        $onlyBugReports = array_diff($bugReportUsers, $reviewUsers, $hardwareSurveyUsers);
        $onlyHardwareSurveys = array_diff($hardwareSurveyUsers, $reviewUsers, $bugReportUsers);
        
        return [
            'review_users_count' => count($reviewUsers),
            'bug_report_users_count' => count($bugReportUsers),
            'hardware_survey_users_count' => count($hardwareSurveyUsers),
            'review_and_bug_reports_count' => count($reviewAndBugReports),
            'review_and_hardware_surveys_count' => count($reviewAndHardwareSurveys),
            'bug_reports_and_hardware_surveys_count' => count($bugReportsAndHardwareSurveys),
            'all_three_count' => count($allThree),
            'only_reviews_count' => count($onlyReviews),
            'only_bug_reports_count' => count($onlyBugReports),
            'only_hardware_surveys_count' => count($onlyHardwareSurveys),
        ];
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