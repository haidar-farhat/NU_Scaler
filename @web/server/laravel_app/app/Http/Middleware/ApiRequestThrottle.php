<?php

namespace App\Http\Middleware;

use Closure;
use Illuminate\Cache\RateLimiter;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Auth;
use Illuminate\Support\Facades\Log;
use Symfony\Component\HttpFoundation\Response;

class ApiRequestThrottle
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
     * @param  int  $maxAttempts
     * @param  int  $decayMinutes
     * @return \Symfony\Component\HttpFoundation\Response
     */
    public function handle(Request $request, Closure $next, int $maxAttempts = 60, int $decayMinutes = 1): Response
    {
        // Modify limits for suspicious IPs
        if ($this->isSuspiciousIP($request->ip())) {
            $maxAttempts = (int) ($maxAttempts * 0.5); // Reduce to 50%
        }

        $key = $this->resolveRequestSignature($request);

        // Check if the request is rate limited
        if ($this->limiter->tooManyAttempts($key, $maxAttempts)) {
            // Log excessive requests
            if ($this->limiter->attempts($key) >= $maxAttempts * 2) {
                Log::channel('security')->warning('Potential API abuse detected', [
                    'ip' => $request->ip(),
                    'user_agent' => $request->header('User-Agent'),
                    'attempts' => $this->limiter->attempts($key),
                    'url' => $request->fullUrl(),
                ]);
            }

            return $this->buildTooManyAttemptsResponse($key, $maxAttempts);
        }

        // Increment the rate limiter
        $this->limiter->hit($key, $decayMinutes * 60);

        $response = $next($request);

        // Add rate limit headers to the response
        return $this->addHeaders(
            $response, $maxAttempts,
            $this->calculateRemainingAttempts($key, $maxAttempts)
        );
    }

    /**
     * Resolve the request signature for the rate limiter.
     *
     * @param  \Illuminate\Http\Request  $request
     * @return string
     */
    protected function resolveRequestSignature(Request $request): string
    {
        // If user is authenticated, use user ID for more granular rate limiting
        if (Auth::check()) {
            $user = Auth::user();

            return sha1($user->id . '|' . $request->ip() . '|' . $request->path());
        }

        // For unauthenticated users, use IP address and path
        return sha1($request->ip() . '|' . $request->path());
    }

    /**
     * Check if an IP address is suspicious.
     *
     * @param  string  $ip
     * @return bool
     */
    protected function isSuspiciousIP(string $ip): bool
    {
        // Check if IP is in suspicious list or has excessive failed attempts
        $key = 'suspicious_ip:' . $ip;
        $failedAttempts = $this->limiter->attempts($key);

        // Calculate suspicion score
        $score = $failedAttempts;

        // Add to suspicious list if not already there
        if ($score >= 10 && !$this->limiter->tooManyAttempts($key, 10)) {
            $this->limiter->hit($key, 60 * 24); // Remember for 24 hours
        }

        return $score >= 10 || $this->limiter->tooManyAttempts($key, 10);
    }

    /**
     * Create a 'too many attempts' response.
     *
     * @param  string  $key
     * @param  int  $maxAttempts
     * @return \Illuminate\Http\Response
     */
    protected function buildTooManyAttemptsResponse(string $key, int $maxAttempts): Response
    {
        $retryAfter = $this->limiter->availableIn($key);

        return response()->json([
            'message' => 'Too many API requests. Please try again later.',
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
        return $maxAttempts - $this->limiter->attempts($key);
    }

    /**
     * Add the rate limiting headers to the response.
     *
     * @param  \Symfony\Component\HttpFoundation\Response  $response
     * @param  int  $maxAttempts
     * @param  int  $remainingAttempts
     * @return \Symfony\Component\HttpFoundation\Response
     */
    protected function addHeaders(Response $response, int $maxAttempts, int $remainingAttempts): Response
    {
        return $response->withHeaders([
            'X-RateLimit-Limit' => $maxAttempts,
            'X-RateLimit-Remaining' => max(0, $remainingAttempts),
        ]);
    }
}
