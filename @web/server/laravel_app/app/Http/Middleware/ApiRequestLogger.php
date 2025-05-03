<?php

namespace App\Http\Middleware;

use Closure;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Log;
use Symfony\Component\HttpFoundation\Response;

class ApiRequestLogger
{
    /**
     * Handle an incoming request.
     *
     * @param  \Illuminate\Http\Request  $request
     * @param  \Closure  $next
     * @return mixed
     */
    public function handle(Request $request, Closure $next)
    {
        // Log incoming API request
        $this->logRequest($request);

        // Process the request
        $response = $next($request);

        // Log the response
        $this->logResponse($request, $response);

        return $response;
    }

    /**
     * Log the request details.
     *
     * @param  \Illuminate\Http\Request  $request
     * @return void
     */
    protected function logRequest(Request $request)
    {
        // Don't log sensitive data like passwords
        $input = $request->except(['password', 'password_confirmation']);

        $logData = [
            'ip' => $request->ip(),
            'method' => $request->method(),
            'url' => $request->fullUrl(),
            'user_agent' => $request->header('User-Agent'),
            'user_id' => $request->user() ? $request->user()->id : 'guest',
            'input' => $input,
        ];

        Log::channel('api')->info('API Request', $logData);
    }

    /**
     * Log the response details.
     *
     * @param  \Illuminate\Http\Request  $request
     * @param  \Symfony\Component\HttpFoundation\Response  $response
     * @return void
     */
    protected function logResponse(Request $request, $response)
    {
        $logData = [
            'ip' => $request->ip(),
            'method' => $request->method(),
            'url' => $request->fullUrl(),
            'status' => $response->getStatusCode(),
            'duration' => microtime(true) - LARAVEL_START,
        ];

        // Log if it's an error response
        if ($response->getStatusCode() >= 400) {
            Log::channel('api')->error('API Error Response', $logData);
        } else {
            Log::channel('api')->info('API Response', $logData);
        }
    }
}
