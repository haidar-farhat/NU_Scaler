<?php

namespace App\Http\Middleware;

use Closure;
use Illuminate\Cache\RateLimiter;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Auth;
use Illuminate\Support\Str;
use Symfony\Component\HttpFoundation\Response;

class ApiRateLimiter
{
    /**
     * The rate limiter instance.
     *
     * @var \Illuminate\Cache\RateLimiter
     */
    protected $limiter;

    /**
     * Create a new middleware instance.
     *
     * @param  \Illuminate\Cache\RateLimiter  $limiter
     * @return void
     */
    public function __construct(RateLimiter $limiter)
    {
        $this->limiter = $limiter;
    }

    /**
     * Handle an incoming request.
     *
     * @param  \Illuminate\Http\Request  $request
     * @param  \Closure  $next
     * @param  string  $limiterType
     * @return \Symfony\Component\HttpFoundation\Response
     */
    public function handle(Request $request, Closure $next, string $limiterType): Response
    {
        // Get rate limits based on limiter type
        [$maxAttempts, $decayMinutes] = $this->getRateLimits($limiterType);

        // Create a unique key for this request
        $key = $this->resolveRequestSignature($request, $limiterType);

        // Check if the request is rate limited
        if ($this->limiter->tooManyAttempts($key, $maxAttempts)) {
            return $this->buildRateLimitResponse($key, $maxAttempts);
        }

        // Increment the rate limiter
        $this->limiter->hit($key, $decayMinutes * 60);

        // Add rate limit headers to the response
        $response = $next($request);
        
        return $this->addRateLimitHeaders(
            $response, $maxAttempts, $this->calculateRemainingAttempts($key, $maxAttempts)
        );
    }

    /**
     * Get rate limits based on limiter type.
     *
     * @param  string  $limiterType
     * @return array
     */
    protected function getRateLimits(string $limiterType): array
    {
        $rateLimits = [
            // Public endpoints
            'public' => [60, 1], // 60 requests per minute
            
            // Feedback endpoints
            'feedback' => [30, 1], // 30 requests per minute
            'reviews' => [20, 1], // 20 requests per minute
            'bug_reports' => [10, 1], // 10 requests per minute
            'hardware_surveys' => [10, 1], // 10 requests per minute
            
            // Auth endpoints
            'auth' => [10, 1], // 10 requests per minute
            'login' => [5, 1], // 5 requests per minute
            'register' => [3, 1], // 3 requests per minute
            
            // Admin endpoints
            'admin' => [120, 1], // 120 requests per minute
            'metrics' => [30, 1], // 30 requests per minute
            
            // Download endpoints
            'downloads' => [5, 1], // 5 requests per minute
        ];

        // Default to public rate limits if specified type not found
        return $rateLimits[$limiterType] ?? [60, 1];
    }

    /**
     * Resolve the request signature for the rate limiter.
     *
     * @param  \Illuminate\Http\Request  $request
     * @param  string  $limiterType
     * @return string
     */
    protected function resolveRequestSignature(Request $request, string $limiterType): string
    {
        // If user is authenticated, use user ID for more granular rate limiting
        if (Auth::check()) {
            $user = Auth::user();
            
            // Admins get their own separate buckets
            if ($user->is_admin) {
                return 'admin|' . $user->id . '|' . $limiterType;
            }
            
            return 'user|' . $user->id . '|' . $limiterType;
        }

        // For unauthenticated users, use IP address
        return 'ip|' . $request->ip() . '|' . $limiterType;
    }

    /**
     * Create a rate limit exceeded response.
     *
     * @param  string  $key
     * @param  int  $maxAttempts
     * @return \Illuminate\Http\JsonResponse
     */
    protected function buildRateLimitResponse(string $key, int $maxAttempts)
    {
        $retryAfter = $this->limiter->availableIn($key);

        return response()->json([
            'message' => 'Too many requests',
            'error' => 'API rate limit exceeded',
            'retry_after' => $retryAfter,
        ], 429)->withHeaders([
            'Retry-After' => $retryAfter,
            'X-RateLimit-Limit' => $maxAttempts,
            'X-RateLimit-Remaining' => 0,
        ]);
    }

    /**
     * Calculate the number of remaining attempts.
     *
     * @param  string  $key
     * @param  int  $maxAttempts
     * @return int
     */
    protected function calculateRemainingAttempts(string $key, int $maxAttempts): int
    {
        return $maxAttempts - $this->limiter->attempts($key) + 1;
    }

    /**
     * Add rate limit headers to the response.
     *
     * @param  \Symfony\Component\HttpFoundation\Response  $response
     * @param  int  $maxAttempts
     * @param  int  $remainingAttempts
     * @return \Symfony\Component\HttpFoundation\Response
     */
    protected function addRateLimitHeaders(Response $response, int $maxAttempts, int $remainingAttempts): Response
    {
        return $response->withHeaders([
            'X-RateLimit-Limit' => $maxAttempts,
            'X-RateLimit-Remaining' => $remainingAttempts,
        ]);
    }
} 