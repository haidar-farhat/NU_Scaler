<?php

namespace App\Http\Controllers\Api\Admin;

use App\Http\Controllers\Controller;
use App\Http\Resources\Api\V1\BugReportResource;
use App\Http\Resources\Api\V1\HardwareSurveyResource;
use App\Http\Resources\Api\V1\ReviewResource;
use App\Models\BugReport;
use App\Models\HardwareSurvey;
use App\Models\Review;
use Illuminate\Database\Eloquent\Builder;
use Illuminate\Http\JsonResponse;
use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;
use Illuminate\Support\Carbon;

class FeedbackController extends Controller
{
    /**
     * Display a paginated, filterable listing of reviews.
     *
     * @param Request $request
     * @return JsonResponse
     */
    public function listReviews(Request $request): JsonResponse
    {
        $query = Review::query();
        
        // Apply filters
        $this->applyReviewFilters($query, $request);
        
        // Sort
        $sortField = $request->input('sort_by', 'created_at');
        $sortDirection = $request->input('sort_direction', 'desc');
        $allowedSortFields = ['id', 'rating', 'created_at'];
        
        if (in_array($sortField, $allowedSortFields)) {
            $query->orderBy($sortField, $sortDirection === 'asc' ? 'asc' : 'desc');
        } else {
            $query->latest();
        }
        
        // Paginate
        $perPage = min($request->input('per_page', 15), 50);
        $reviews = $query->paginate($perPage);
        
        return ReviewResource::collection($reviews)
            ->additional([
                'meta' => [
                    'average_rating' => $this->getReviewsAverageRating($request),
                    'total_count' => $this->getReviewsTotalCount($request),
                    'filters' => $this->getActiveFilters($request),
                ],
            ])
            ->response();
    }
    
    /**
     * Display details of a specific review.
     *
     * @param Review $review
     * @return JsonResponse
     */
    public function showReview(Review $review): JsonResponse
    {
        return (new ReviewResource($review))
            ->additional([
                'user_details' => $review->user ? [
                    'uuid' => $review->user->uuid,
                    'name' => $review->user->name,
                ] : null,
                'other_reviews' => $review->user ? 
                    ReviewResource::collection(
                        Review::where('user_uuid', $review->user_uuid)
                            ->where('id', '!=', $review->id)
                            ->latest()
                            ->limit(5)
                            ->get()
                    ) : [],
            ])
            ->response();
    }

    /**
     * Display a paginated, filterable listing of bug reports.
     *
     * @param Request $request
     * @return JsonResponse
     */
    public function listBugReports(Request $request): JsonResponse
    {
        $query = BugReport::query();
        
        // Apply filters
        $this->applyBugReportFilters($query, $request);
        
        // Sort
        $sortField = $request->input('sort_by', 'created_at');
        $sortDirection = $request->input('sort_direction', 'desc');
        $allowedSortFields = ['id', 'severity', 'created_at'];
        
        if (in_array($sortField, $allowedSortFields)) {
            $query->orderBy($sortField, $sortDirection === 'asc' ? 'asc' : 'desc');
        } else {
            $query->latest();
        }
        
        // Paginate
        $perPage = min($request->input('per_page', 15), 50);
        $bugReports = $query->paginate($perPage);

        return BugReportResource::collection($bugReports)
            ->additional([
                'meta' => [
                    'severity_counts' => $this->getBugReportSeverityCounts($request),
                    'total_count' => $this->getBugReportsTotalCount($request),
                    'filters' => $this->getActiveFilters($request),
                ],
            ])
            ->response();
    }
    
    /**
     * Display details of a specific bug report.
     *
     * @param BugReport $bugReport
     * @return JsonResponse
     */
    public function showBugReport(BugReport $bugReport): JsonResponse
    {
        return (new BugReportResource($bugReport))
            ->additional([
                'user_details' => $bugReport->user ? [
                    'uuid' => $bugReport->user->uuid,
                    'name' => $bugReport->user->name,
                ] : null,
                'other_bug_reports' => $bugReport->user ? 
                    BugReportResource::collection(
                        BugReport::where('user_uuid', $bugReport->user_uuid)
                            ->where('id', '!=', $bugReport->id)
                            ->latest()
                            ->limit(5)
                            ->get()
                    ) : [],
            ])
            ->response();
    }

    /**
     * Display a paginated, filterable listing of hardware surveys.
     *
     * @param Request $request
     * @return JsonResponse
     */
    public function listHardwareSurveys(Request $request): JsonResponse
    {
        $query = HardwareSurvey::query();
        
        // Apply filters
        $this->applyHardwareSurveyFilters($query, $request);
        
        // Sort
        $sortField = $request->input('sort_by', 'created_at');
        $sortDirection = $request->input('sort_direction', 'desc');
        $allowedSortFields = ['id', 'os', 'gpu', 'ram', 'created_at'];
        
        if (in_array($sortField, $allowedSortFields)) {
            $query->orderBy($sortField, $sortDirection === 'asc' ? 'asc' : 'desc');
        } else {
            $query->latest();
        }
        
        // Paginate
        $perPage = min($request->input('per_page', 15), 50);
        $hardwareSurveys = $query->paginate($perPage);

        return HardwareSurveyResource::collection($hardwareSurveys)
            ->additional([
                'meta' => [
                    'os_distribution' => $this->getOsDistribution($request),
                    'total_count' => $this->getHardwareSurveysTotalCount($request),
                    'filters' => $this->getActiveFilters($request),
                ],
            ])
            ->response();
    }
    
    /**
     * Display details of a specific hardware survey.
     *
     * @param HardwareSurvey $hardwareSurvey
     * @return JsonResponse
     */
    public function showHardwareSurvey(HardwareSurvey $hardwareSurvey): JsonResponse
    {
        return (new HardwareSurveyResource($hardwareSurvey))
            ->additional([
                'user_details' => $hardwareSurvey->user ? [
                    'uuid' => $hardwareSurvey->user->uuid,
                    'name' => $hardwareSurvey->user->name,
                ] : null,
                'other_hardware_surveys' => $hardwareSurvey->user ? 
                    HardwareSurveyResource::collection(
                        HardwareSurvey::where('user_uuid', $hardwareSurvey->user_uuid)
                            ->where('id', '!=', $hardwareSurvey->id)
                            ->latest()
                            ->limit(5)
                            ->get()
                    ) : [],
            ])
            ->response();
    }
    
    /**
     * Apply filtering to the reviews query.
     * 
     * @param Builder $query
     * @param Request $request
     * @return void
     */
    private function applyReviewFilters(Builder $query, Request $request): void
    {
        // Filter by rating
        if ($request->has('rating')) {
            $query->where('rating', $request->input('rating'));
        }
        
        // Filter by rating range
        if ($request->has('min_rating')) {
            $query->where('rating', '>=', $request->input('min_rating'));
        }
        
        if ($request->has('max_rating')) {
            $query->where('rating', '<=', $request->input('max_rating'));
        }
        
        // Filter by date range
        $this->applyDateRangeFilter($query, $request);
        
        // Filter by user
        if ($request->has('user_uuid')) {
            $query->where('user_uuid', $request->input('user_uuid'));
        }
        
        // Search in comment content
        if ($request->has('search')) {
            $searchTerm = $request->input('search');
            $query->where(function ($q) use ($searchTerm) {
                $q->where('comment', 'like', "%{$searchTerm}%")
                  ->orWhere('name', 'like', "%{$searchTerm}%")
                  ->orWhere('email', 'like', "%{$searchTerm}%");
            });
        }
    }
    
    /**
     * Apply filtering to the bug reports query.
     * 
     * @param Builder $query
     * @param Request $request
     * @return void
     */
    private function applyBugReportFilters(Builder $query, Request $request): void
    {
        // Filter by severity
        if ($request->has('severity')) {
            $query->where('severity', $request->input('severity'));
        }
        
        // Filter by specific severities
        if ($request->has('severities')) {
            $severities = explode(',', $request->input('severities'));
            $query->whereIn('severity', $severities);
        }
        
        // Filter by date range
        $this->applyDateRangeFilter($query, $request);
        
        // Filter by user
        if ($request->has('user_uuid')) {
            $query->where('user_uuid', $request->input('user_uuid'));
        }
        
        // Search in description
        if ($request->has('search')) {
            $searchTerm = $request->input('search');
            $query->where(function ($q) use ($searchTerm) {
                $q->where('description', 'like', "%{$searchTerm}%")
                  ->orWhere('steps_to_reproduce', 'like', "%{$searchTerm}%");
            });
        }
    }
    
    /**
     * Apply filtering to the hardware surveys query.
     * 
     * @param Builder $query
     * @param Request $request
     * @return void
     */
    private function applyHardwareSurveyFilters(Builder $query, Request $request): void
    {
        // Filter by operating system
        if ($request->has('os')) {
            $query->where('os', 'like', '%' . $request->input('os') . '%');
        }
        
        // Filter by GPU
        if ($request->has('gpu')) {
            $query->where('gpu', 'like', '%' . $request->input('gpu') . '%');
        }
        
        // Filter by CPU
        if ($request->has('cpu')) {
            $query->where('cpu', 'like', '%' . $request->input('cpu') . '%');
        }
        
        // Filter by RAM
        if ($request->has('min_ram')) {
            $query->where('ram', '>=', $request->input('min_ram'));
        }
        
        if ($request->has('max_ram')) {
            $query->where('ram', '<=', $request->input('max_ram'));
        }
        
        // Filter by date range
        $this->applyDateRangeFilter($query, $request);
        
        // Filter by user
        if ($request->has('user_uuid')) {
            $query->where('user_uuid', $request->input('user_uuid'));
        }
        
        // Search in any field
        if ($request->has('search')) {
            $searchTerm = $request->input('search');
            $query->where(function ($q) use ($searchTerm) {
                $q->where('os', 'like', "%{$searchTerm}%")
                  ->orWhere('gpu', 'like', "%{$searchTerm}%")
                  ->orWhere('cpu', 'like', "%{$searchTerm}%")
                  ->orWhere('resolution', 'like', "%{$searchTerm}%");
            });
        }
    }
    
    /**
     * Apply date range filter to any query.
     * 
     * @param Builder $query
     * @param Request $request
     * @return void
     */
    private function applyDateRangeFilter(Builder $query, Request $request): void
    {
        // Filter by start date
        if ($request->has('start_date')) {
            $query->where('created_at', '>=', Carbon::parse($request->input('start_date'))->startOfDay());
        }
        
        // Filter by end date
        if ($request->has('end_date')) {
            $query->where('created_at', '<=', Carbon::parse($request->input('end_date'))->endOfDay());
        }
        
        // Filter by predefined timeframe
        if ($request->has('timeframe')) {
            $timeframe = $request->input('timeframe');
            $startDate = null;
            
            switch ($timeframe) {
                case 'today':
                    $startDate = Carbon::today();
                    break;
                case 'yesterday':
                    $startDate = Carbon::yesterday();
                    break;
                case 'week':
                    $startDate = Carbon::now()->subWeek();
                    break;
                case 'month':
                    $startDate = Carbon::now()->subMonth();
                    break;
                case 'quarter':
                    $startDate = Carbon::now()->subQuarter();
                    break;
                case 'year':
                    $startDate = Carbon::now()->subYear();
                    break;
            }
            
            if ($startDate) {
                $query->where('created_at', '>=', $startDate);
            }
        }
    }
    
    /**
     * Get active filters from the request.
     * 
     * @param Request $request
     * @return array
     */
    private function getActiveFilters(Request $request): array
    {
        $filters = [];
        
        $possibleFilters = [
            'rating', 'min_rating', 'max_rating', 'severity', 'severities',
            'os', 'gpu', 'cpu', 'min_ram', 'max_ram', 'user_uuid',
            'start_date', 'end_date', 'timeframe', 'search',
        ];
        
        foreach ($possibleFilters as $filter) {
            if ($request->has($filter)) {
                $filters[$filter] = $request->input($filter);
            }
        }
        
        return $filters;
    }
    
    /**
     * Get bug report severity counts with filters applied.
     * 
     * @param Request $request
     * @return array
     */
    private function getBugReportSeverityCounts(Request $request): array
    {
        $query = BugReport::query();
        
        // Apply all filters except severity itself
        $this->applyDateRangeFilter($query, $request);
        
        if ($request->has('user_uuid')) {
            $query->where('user_uuid', $request->input('user_uuid'));
        }
        
        if ($request->has('search')) {
            $searchTerm = $request->input('search');
            $query->where(function ($q) use ($searchTerm) {
                $q->where('description', 'like', "%{$searchTerm}%")
                  ->orWhere('steps_to_reproduce', 'like', "%{$searchTerm}%");
            });
        }
        
        $severityCounts = $query->select('severity', \DB::raw('count(*) as count'))
            ->groupBy('severity')
            ->get()
            ->pluck('count', 'severity')
            ->toArray();
        
        // Ensure all severity levels are represented
        $severityLevels = ['low', 'medium', 'high', 'critical'];
        $completeDistribution = [];
        
        foreach ($severityLevels as $level) {
            $completeDistribution[$level] = $severityCounts[$level] ?? 0;
        }
        
        return $completeDistribution;
    }
    
    /**
     * Get reviews average rating with filters applied.
     * 
     * @param Request $request
     * @return float
     */
    private function getReviewsAverageRating(Request $request): float
    {
        $query = Review::query();
        
        // Apply all filters except rating itself
        $this->applyDateRangeFilter($query, $request);
        
        if ($request->has('user_uuid')) {
            $query->where('user_uuid', $request->input('user_uuid'));
        }
        
        if ($request->has('search')) {
            $searchTerm = $request->input('search');
            $query->where(function ($q) use ($searchTerm) {
                $q->where('comment', 'like', "%{$searchTerm}%")
                  ->orWhere('name', 'like', "%{$searchTerm}%")
                  ->orWhere('email', 'like', "%{$searchTerm}%");
            });
        }
        
        return round($query->avg('rating') ?? 0, 2);
    }
    
    /**
     * Get OS distribution with filters applied.
     * 
     * @param Request $request
     * @return array
     */
    private function getOsDistribution(Request $request): array
    {
        $query = HardwareSurvey::query();
        
        // Apply filters
        $this->applyDateRangeFilter($query, $request);
        
        if ($request->has('gpu')) {
            $query->where('gpu', 'like', '%' . $request->input('gpu') . '%');
        }
        
        if ($request->has('cpu')) {
            $query->where('cpu', 'like', '%' . $request->input('cpu') . '%');
        }
        
        if ($request->has('min_ram')) {
            $query->where('ram', '>=', $request->input('min_ram'));
        }
        
        if ($request->has('max_ram')) {
            $query->where('ram', '<=', $request->input('max_ram'));
        }
        
        if ($request->has('user_uuid')) {
            $query->where('user_uuid', $request->input('user_uuid'));
        }
        
        $osDistribution = $query->select('os', \DB::raw('count(*) as count'))
            ->groupBy('os')
            ->orderBy('count', 'desc')
            ->get()
            ->pluck('count', 'os')
            ->toArray();
        
        return $osDistribution;
    }
    
    /**
     * Get total number of reviews with filters applied.
     * 
     * @param Request $request
     * @return int
     */
    private function getReviewsTotalCount(Request $request): int
    {
        $query = Review::query();
        $this->applyReviewFilters($query, $request);
        return $query->count();
    }
    
    /**
     * Get total number of bug reports with filters applied.
     * 
     * @param Request $request
     * @return int
     */
    private function getBugReportsTotalCount(Request $request): int
    {
        $query = BugReport::query();
        $this->applyBugReportFilters($query, $request);
        return $query->count();
    }
    
    /**
     * Get total number of hardware surveys with filters applied.
     * 
     * @param Request $request
     * @return int
     */
    private function getHardwareSurveysTotalCount(Request $request): int
    {
        $query = HardwareSurvey::query();
        $this->applyHardwareSurveyFilters($query, $request);
        return $query->count();
    }

    /**
     * Get trending topics from feedback.
     *
     * @param Request $request
     * @return JsonResponse
     */
    public function trendingTopics(Request $request): JsonResponse
    {
        $timeframe = $request->input('timeframe', 'month');
        $limit = min(20, max(5, $request->input('limit', 10)));
        
        // Determine start date based on timeframe
        $startDate = null;
        
        switch ($timeframe) {
            case 'day':
                $startDate = Carbon::today();
                break;
            case 'week':
                $startDate = Carbon::now()->subWeek();
                break;
            case 'month':
                $startDate = Carbon::now()->subMonth();
                break;
            case 'quarter':
                $startDate = Carbon::now()->subQuarter();
                break;
            case 'year':
                $startDate = Carbon::now()->subYear();
                break;
        }
        
        // Extract common keywords from reviews
        $reviewKeywords = $this->extractKeywordsFromText(
            Review::query()
                ->when($startDate, function ($query) use ($startDate) {
                    $query->where('created_at', '>=', $startDate);
                })
                ->select('comment')
                ->get()
                ->pluck('comment')
                ->toArray()
        );
        
        // Extract common keywords from bug reports
        $bugReportKeywords = $this->extractKeywordsFromText(
            BugReport::query()
                ->when($startDate, function ($query) use ($startDate) {
                    $query->where('created_at', '>=', $startDate);
                })
                ->select('description', 'steps_to_reproduce')
                ->get()
                ->map(function ($item) {
                    return $item->description . ' ' . $item->steps_to_reproduce;
                })
                ->toArray()
        );
        
        // Extract keywords by review rating
        $keywordsByRating = [];
        for ($rating = 1; $rating <= 5; $rating++) {
            $keywordsByRating[$rating] = $this->extractKeywordsFromText(
                Review::query()
                    ->where('rating', $rating)
                    ->when($startDate, function ($query) use ($startDate) {
                        $query->where('created_at', '>=', $startDate);
                    })
                    ->select('comment')
                    ->get()
                    ->pluck('comment')
                    ->toArray()
            );
        }
        
        // Extract keywords by bug severity
        $keywordsBySeverity = [];
        foreach (['low', 'medium', 'high', 'critical'] as $severity) {
            $keywordsBySeverity[$severity] = $this->extractKeywordsFromText(
                BugReport::query()
                    ->where('severity', $severity)
                    ->when($startDate, function ($query) use ($startDate) {
                        $query->where('created_at', '>=', $startDate);
                    })
                    ->select('description', 'steps_to_reproduce')
                    ->get()
                    ->map(function ($item) {
                        return $item->description . ' ' . $item->steps_to_reproduce;
                    })
                    ->toArray()
            );
        }
        
        return response()->json([
            'trending_topics' => [
                'reviews' => array_slice($reviewKeywords, 0, $limit),
                'bug_reports' => array_slice($bugReportKeywords, 0, $limit),
                'by_rating' => array_map(function ($keywords) use ($limit) {
                    return array_slice($keywords, 0, $limit);
                }, $keywordsByRating),
                'by_severity' => array_map(function ($keywords) use ($limit) {
                    return array_slice($keywords, 0, $limit);
                }, $keywordsBySeverity),
            ],
            'timeframe' => $timeframe,
        ]);
    }
    
    /**
     * Extract keywords from an array of text.
     *
     * @param array $texts
     * @return array
     */
    private function extractKeywordsFromText(array $texts): array
    {
        $combinedText = implode(' ', $texts);
        
        // Remove common stop words
        $stopWords = ['a', 'an', 'the', 'and', 'or', 'but', 'is', 'are', 'was', 'were', 
                     'to', 'of', 'in', 'for', 'with', 'on', 'at', 'by', 'this', 'that',
                     'it', 'not', 'be', 'have', 'has', 'had', 'do', 'does', 'did',
                     'i', 'my', 'me', 'mine', 'you', 'your', 'yours', 'he', 'she', 'him',
                     'her', 'his', 'they', 'them', 'their', 'we', 'us', 'our'];
                     
        // Remove punctuation and convert to lowercase
        $cleanedText = strtolower(preg_replace('/[^\p{L}\p{N}\s]/u', '', $combinedText));
        
        // Split into words
        $words = preg_split('/\s+/', $cleanedText, -1, PREG_SPLIT_NO_EMPTY);
        
        // Filter out stop words and words less than 3 characters
        $words = array_filter($words, function ($word) use ($stopWords) {
            return !in_array($word, $stopWords) && strlen($word) >= 3;
        });
        
        // Count word frequencies
        $wordCounts = array_count_values($words);
        
        // Sort by frequency, descending
        arsort($wordCounts);
        
        // Convert to proper format
        $keywords = [];
        foreach ($wordCounts as $word => $count) {
            $keywords[] = [
                'keyword' => $word,
                'count' => $count,
            ];
        }
        
        return $keywords;
    }
} 