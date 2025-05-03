<?php

namespace App\Http\Middleware;

use Closure;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Auth;
use Illuminate\Support\Facades\Log;
use Symfony\Component\HttpFoundation\Response;

class ApiRequestLogger
{
    /**
     * Handle an incoming request.
     *
     * @param  \Illuminate\Http\Request  $request
     * @param  \Closure  $next
     * @return \Symfony\Component\HttpFoundation\Response
     */
    public function handle(Request $request, Closure $next): Response
    {
        // Get the response
        $response = $next($request);
        
        // Skip logging for non-API requests or health checks
        if (!$this->shouldLogRequest($request)) {
            return $response;
        }

        // Prepare log data
        $logData = [
            'method' => $request->method(),
            'url' => $request->fullUrl(),
            'ip' => $request->ip(),
            'user_agent' => $request->header('User-Agent'),
            'status_code' => $response->getStatusCode(),
            'duration_ms' => round((microtime(true) - LARAVEL_START) * 1000, 2),
        ];

        // Add user information if available
        if (Auth::check()) {
            $user = Auth::user();
            $logData['user_id'] = $user->id;
            $logData['user_name'] = $user->name;
            $logData['is_admin'] = $user->is_admin;
        }

        // Add request parameters (sanitized)
        $safeParams = $this->sanitizeRequestParams($request);
        if (!empty($safeParams)) {
            $logData['params'] = $safeParams;
        }

        // Determine log level based on response status
        $statusCode = $response->getStatusCode();
        
        if ($statusCode >= 500) {
            Log::channel('api')->error('API Request Error', $logData);
        } elseif ($statusCode >= 400) {
            Log::channel('api')->warning('API Request Warning', $logData);
        } else {
            Log::channel('api')->info('API Request', $logData);
        }

        return $response;
    }

    /**
     * Determine if the request should be logged.
     *
     * @param  \Illuminate\Http\Request  $request
     * @return bool
     */
    protected function shouldLogRequest(Request $request): bool
    {
        // Skip health checks and monitoring endpoints
        if ($request->is('api/health*', 'api/ping', 'api/status')) {
            return false;
        }

        // Always log API requests
        return $request->is('api/*');
    }

    /**
     * Sanitize request parameters for logging.
     *
     * @param  \Illuminate\Http\Request  $request
     * @return array
     */
    protected function sanitizeRequestParams(Request $request): array
    {
        // Get all input parameters
        $params = $request->except([
            'password', 'password_confirmation', 'token', 'authorization',
            'api_token', 'key', 'secret', 'credit_card', 'card_number'
        ]);

        // Remove binary data or large text fields
        foreach ($params as $key => $value) {
            if (is_string($value) && strlen($value) > 500) {
                $params[$key] = '[LARGE CONTENT - ' . strlen($value) . ' bytes]';
            }
        }

        return $params;
    }
} 