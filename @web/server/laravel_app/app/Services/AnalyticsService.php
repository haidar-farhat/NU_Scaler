<?php

namespace App\Services;

use App\Models\BugReport;
use App\Models\HardwareSurvey;
use App\Models\Review;
use Illuminate\Support\Facades\Cache;
use Illuminate\Support\Facades\DB;

class AnalyticsService
{
    /**
     * Process feedback for analytics.
     *
     * @param mixed $feedback
     * @param string $type
     * @return void
     */
    public function processFeedback($feedback, string $type): void
    {
        // Determine the appropriate method based on feedback type
        match($type) {
            'review' => $this->processReview($feedback),
            'bug-report' => $this->processBugReport($feedback),
            'hardware-survey' => $this->processHardwareSurvey($feedback),
            default => null,
        };

        // Clear caches for analytics
        $this->clearAnalyticsCache();
    }

    /**
     * Process a review for analytics.
     *
     * @param \App\Models\Review $review
     * @return void
     */
    protected function processReview(Review $review): void
    {
        // Update average rating
        $this->updateAverageRating();

        // Update ratings distribution
        $this->updateRatingsDistribution();

        // Process sentiment analysis if text is present
        if (!empty($review->comment)) {
            $this->processSentimentAnalysis($review->comment);
        }
    }

    /**
     * Process a bug report for analytics.
     *
     * @param \App\Models\BugReport $bugReport
     * @return void
     */
    protected function processBugReport(BugReport $bugReport): void
    {
        // Update bug categories distribution
        $this->updateBugCategoriesDistribution();

        // Update bug severity statistics
        $this->updateBugSeverityStatistics();
    }

    /**
     * Process a hardware survey for analytics.
     *
     * @param \App\Models\HardwareSurvey $hardwareSurvey
     * @return void
     */
    protected function processHardwareSurvey(HardwareSurvey $hardwareSurvey): void
    {
        // Update hardware statistics
        $this->updateHardwareStatistics();
    }

    /**
     * Update average rating.
     *
     * @return void
     */
    protected function updateAverageRating(): void
    {
        $averageRating = Review::avg('rating');
        Cache::put('analytics.average_rating', $averageRating, now()->addDay());
    }

    /**
     * Update ratings distribution.
     *
     * @return void
     */
    protected function updateRatingsDistribution(): void
    {
        $distribution = Review::select('rating', DB::raw('count(*) as count'))
            ->groupBy('rating')
            ->orderBy('rating')
            ->get()
            ->pluck('count', 'rating')
            ->toArray();

        Cache::put('analytics.ratings_distribution', $distribution, now()->addDay());
    }

    /**
     * Process sentiment analysis on text.
     *
     * @param string $text
     * @return void
     */
    protected function processSentimentAnalysis(string $text): void
    {
        // Simple keyword-based sentiment analysis
        $positiveWords = ['great', 'love', 'excellent', 'amazing', 'good', 'fantastic', 'awesome'];
        $negativeWords = ['bad', 'poor', 'terrible', 'horrible', 'hate', 'disappointing', 'awful'];

        $text = strtolower($text);
        $positiveCount = 0;
        $negativeCount = 0;

        foreach ($positiveWords as $word) {
            $positiveCount += substr_count($text, $word);
        }

        foreach ($negativeWords as $word) {
            $negativeCount += substr_count($text, $word);
        }

        // Update sentiment counters
        $currentPositive = Cache::get('analytics.sentiment.positive', 0);
        $currentNegative = Cache::get('analytics.sentiment.negative', 0);

        Cache::put('analytics.sentiment.positive', $currentPositive + $positiveCount, now()->addWeek());
        Cache::put('analytics.sentiment.negative', $currentNegative + $negativeCount, now()->addWeek());
    }

    /**
     * Update bug categories distribution.
     *
     * @return void
     */
    protected function updateBugCategoriesDistribution(): void
    {
        $distribution = BugReport::select('category', DB::raw('count(*) as count'))
            ->groupBy('category')
            ->orderBy('category')
            ->get()
            ->pluck('count', 'category')
            ->toArray();

        Cache::put('analytics.bug_categories', $distribution, now()->addDay());
    }

    /**
     * Update bug severity statistics.
     *
     * @return void
     */
    protected function updateBugSeverityStatistics(): void
    {
        $distribution = BugReport::select('severity', DB::raw('count(*) as count'))
            ->groupBy('severity')
            ->orderBy('severity')
            ->get()
            ->pluck('count', 'severity')
            ->toArray();

        Cache::put('analytics.bug_severity', $distribution, now()->addDay());
    }

    /**
     * Update hardware statistics.
     *
     * @return void
     */
    protected function updateHardwareStatistics(): void
    {
        // GPU Distribution
        $gpuDistribution = HardwareSurvey::select('gpu_model', DB::raw('count(*) as count'))
            ->groupBy('gpu_model')
            ->orderByDesc('count')
            ->limit(10)
            ->get()
            ->pluck('count', 'gpu_model')
            ->toArray();

        Cache::put('analytics.hardware.gpu_distribution', $gpuDistribution, now()->addDay());

        // CPU Distribution
        $cpuDistribution = HardwareSurvey::select('cpu_model', DB::raw('count(*) as count'))
            ->groupBy('cpu_model')
            ->orderByDesc('count')
            ->limit(10)
            ->get()
            ->pluck('count', 'cpu_model')
            ->toArray();

        Cache::put('analytics.hardware.cpu_distribution', $cpuDistribution, now()->addDay());

        // OS Distribution
        $osDistribution = HardwareSurvey::select('os', DB::raw('count(*) as count'))
            ->groupBy('os')
            ->orderByDesc('count')
            ->get()
            ->pluck('count', 'os')
            ->toArray();

        Cache::put('analytics.hardware.os_distribution', $osDistribution, now()->addDay());
    }

    /**
     * Clear analytics cache.
     *
     * @return void
     */
    protected function clearAnalyticsCache(): void
    {
        Cache::forget('admin.dashboard.metrics');
        Cache::forget('api.metrics.summary');
    }

    /**
     * Get review metrics.
     *
     * @return array
     */
    public function getReviewMetrics(): array
    {
        return [
            'average_rating' => Cache::remember('analytics.average_rating', now()->addHour(),
                function () {
                    return Review::avg('rating');
                }
            ),
            'total_reviews' => Cache::remember('analytics.total_reviews', now()->addHour(),
                function () {
                    return Review::count();
                }
            ),
            'ratings_distribution' => Cache::remember('analytics.ratings_distribution', now()->addHour(),
                function () {
                    return Review::select('rating', DB::raw('count(*) as count'))
                        ->groupBy('rating')
                        ->orderBy('rating')
                        ->get()
                        ->pluck('count', 'rating')
                        ->toArray();
                }
            ),
            'sentiment' => [
                'positive' => Cache::get('analytics.sentiment.positive', 0),
                'negative' => Cache::get('analytics.sentiment.negative', 0),
            ],
        ];
    }

    /**
     * Get bug report metrics.
     *
     * @return array
     */
    public function getBugReportMetrics(): array
    {
        return [
            'total_bugs' => Cache::remember('analytics.total_bugs', now()->addHour(),
                function () {
                    return BugReport::count();
                }
            ),
            'categories_distribution' => Cache::remember('analytics.bug_categories', now()->addHour(),
                function () {
                    return BugReport::select('category', DB::raw('count(*) as count'))
                        ->groupBy('category')
                        ->orderBy('category')
                        ->get()
                        ->pluck('count', 'category')
                        ->toArray();
                }
            ),
            'severity_distribution' => Cache::remember('analytics.bug_severity', now()->addHour(),
                function () {
                    return BugReport::select('severity', DB::raw('count(*) as count'))
                        ->groupBy('severity')
                        ->orderBy('severity')
                        ->get()
                        ->pluck('count', 'severity')
                        ->toArray();
                }
            ),
        ];
    }

    /**
     * Get hardware survey metrics.
     *
     * @return array
     */
    public function getHardwareSurveyMetrics(): array
    {
        return [
            'total_surveys' => Cache::remember('analytics.total_surveys', now()->addHour(),
                function () {
                    return HardwareSurvey::count();
                }
            ),
            'gpu_distribution' => Cache::remember('analytics.hardware.gpu_distribution', now()->addHour(),
                function () {
                    return HardwareSurvey::select('gpu_model', DB::raw('count(*) as count'))
                        ->groupBy('gpu_model')
                        ->orderByDesc('count')
                        ->limit(10)
                        ->get()
                        ->pluck('count', 'gpu_model')
                        ->toArray();
                }
            ),
            'cpu_distribution' => Cache::remember('analytics.hardware.cpu_distribution', now()->addHour(),
                function () {
                    return HardwareSurvey::select('cpu_model', DB::raw('count(*) as count'))
                        ->groupBy('cpu_model')
                        ->orderByDesc('count')
                        ->limit(10)
                        ->get()
                        ->pluck('count', 'cpu_model')
                        ->toArray();
                }
            ),
            'os_distribution' => Cache::remember('analytics.hardware.os_distribution', now()->addHour(),
                function () {
                    return HardwareSurvey::select('os', DB::raw('count(*) as count'))
                        ->groupBy('os')
                        ->orderByDesc('count')
                        ->get()
                        ->pluck('count', 'os')
                        ->toArray();
                }
            ),
            'average_memory' => Cache::remember('analytics.hardware.average_memory', now()->addHour(),
                function () {
                    return HardwareSurvey::avg('ram_size');
                }
            ),
        ];
    }

    /**
     * Get all feedback metrics.
     *
     * @return array
     */
    public function getAllMetrics(): array
    {
        return Cache::remember('api.metrics.summary', now()->addHour(), function () {
            return [
                'reviews' => $this->getReviewMetrics(),
                'bug_reports' => $this->getBugReportMetrics(),
                'hardware_surveys' => $this->getHardwareSurveyMetrics(),
            ];
        });
    }
}
