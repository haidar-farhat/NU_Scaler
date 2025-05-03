<?php

namespace App\Http\Middleware;

use Closure;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Cache;
use Symfony\Component\HttpFoundation\Response;

class CacheResponse
{
    /**
     * Cache TTL in seconds.
     *
     * @var int
     */
    protected $ttl = 60;

    /**
     * Handle an incoming request.
     *
     * @param  \Illuminate\Http\Request  $request
     * @param  \Closure  $next
     * @param  int|null  $ttl
     * @return mixed
     */
    public function handle(Request $request, Closure $next, $ttl = null)
    {
        // Don't cache if the request is not a GET request
        if (!$request->isMethod('GET')) {
            return $next($request);
        }

        // Don't cache if the user is authenticated
        if ($request->user()) {
            return $next($request);
        }

        // Set TTL if provided
        if (!is_null($ttl)) {
            $this->ttl = (int) $ttl;
        }

        // Generate a cache key based on the full URL
        $key = $this->getCacheKey($request);

        // Check if we have a cached response
        if (Cache::store('api_responses')->has($key)) {
            return Cache::store('api_responses')->get($key);
        }

        // Process the request
        $response = $next($request);

        // Cache the response if it's successful
        if ($this->shouldCache($response)) {
            Cache::store('api_responses')->put($key, $response, $this->ttl);
        }

        return $response;
    }

    /**
     * Generate a cache key for the request.
     *
     * @param  \Illuminate\Http\Request  $request
     * @return string
     */
    protected function getCacheKey(Request $request): string
    {
        return 'api_response:' . md5($request->fullUrl());
    }

    /**
     * Determine if the response should be cached.
     *
     * @param  \Symfony\Component\HttpFoundation\Response  $response
     * @return bool
     */
    protected function shouldCache(Response $response): bool
    {
        // Only cache successful responses
        if (!$response->isSuccessful()) {
            return false;
        }

        // Get status code
        $statusCode = $response->getStatusCode();

        // Only cache 200 OK responses
        if ($statusCode !== 200) {
            return false;
        }

        return true;
    }
}
