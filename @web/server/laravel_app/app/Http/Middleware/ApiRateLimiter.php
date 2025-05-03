<?php

namespace App\Http\Middleware;

use Closure;
use Illuminate\Cache\RateLimiter;
use Illuminate\Http\Request;
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
     * Create a new rate limiter middleware.
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
     * @param  string  $type
     * @return \Symfony\Component\HttpFoundation\Response
     */
    public function handle(Request $request, Closure $next, $type = 'api')
    {
        $key = $this->resolveRequestSignature($request, $type);

        $maxAttempts = $this->getMaxAttemptsByType($type);
        $decayMinutes = $this->getDecayMinutesByType($type);

        if ($this->limiter->tooManyAttempts($key, $maxAttempts)) {
            return $this->buildResponse($key, $maxAttempts);
        }

        $this->limiter->hit($key, $decayMinutes * 60);

        $response = $next($request);

        return $this->addRateLimitHeaders(
            $response, $maxAttempts, $this->calculateRemainingAttempts($key, $maxAttempts)
        );
    }

    /**
     * Resolve the request signature for the rate limiter.
     *
     * @param  \Illuminate\Http\Request  $request
     * @param  string  $type
     * @return string
     */
    protected function resolveRequestSignature(Request $request, $type)
    {
        $user = $request->user();

        return sha1($type.'|'.($user ? $user->id : $request->ip()));
    }

    /**
     * Get the maximum number of attempts based on type.
     *
     * @param  string  $type
     * @return int
     */
    protected function getMaxAttemptsByType($type)
    {
        return match($type) {
            'admin' => 60,     // 60 requests per 15 minutes for admin
            'downloads' => 10, // 10 downloads per hour
            'feedback' => 5,   // 5 feedback submissions per hour
            default => 60,     // Default 60 requests per minute
        };
    }

    /**
     * Get decay minutes based on type.
     *
     * @param  string  $type
     * @return int
     */
    protected function getDecayMinutesByType($type)
    {
        return match($type) {
            'admin' => 15,     // 15 minutes
            'downloads' => 60, // 60 minutes
            'feedback' => 60,  // 60 minutes
            default => 1,      // Default 1 minute
        };
    }

    /**
     * Create a 'too many attempts' response.
     *
     * @param  string  $key
     * @param  int  $maxAttempts
     * @return \Symfony\Component\HttpFoundation\Response
     */
    protected function buildResponse($key, $maxAttempts)
    {
        $retryAfter = $this->limiter->availableIn($key);

        return response()->json([
            'message' => 'Too many requests. Please try again later.',
            'retry_after' => $retryAfter,
        ], Response::HTTP_TOO_MANY_REQUESTS)->withHeaders([
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
    protected function calculateRemainingAttempts($key, $maxAttempts)
    {
        return $maxAttempts - $this->limiter->attempts($key) + 1;
    }

    /**
     * Add the rate limit headers to the response.
     *
     * @param  \Symfony\Component\HttpFoundation\Response  $response
     * @param  int  $maxAttempts
     * @param  int  $remainingAttempts
     * @return \Symfony\Component\HttpFoundation\Response
     */
    protected function addRateLimitHeaders($response, $maxAttempts, $remainingAttempts)
    {
        return $response->withHeaders([
            'X-RateLimit-Limit' => $maxAttempts,
            'X-RateLimit-Remaining' => $remainingAttempts,
        ]);
    }
}
