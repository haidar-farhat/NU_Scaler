<?php

namespace App\Http\Middleware;

use Closure;
use Illuminate\Http\Request;
use Symfony\Component\HttpFoundation\Response;

class PreventRequestsWithoutAcceptJson
{
    /**
     * Ensure that all requests to the API include the Accept: application/json header.
     *
     * @param  \Illuminate\Http\Request  $request
     * @param  \Closure  $next
     * @return \Symfony\Component\HttpFoundation\Response
     */
    public function handle(Request $request, Closure $next): Response
    {
        if (!$request->expectsJson()) {
            return response()->json([
                'message' => 'API requests must include Accept: application/json header',
                'status' => 'error',
                'code' => 406,
            ], 406);
        }

        return $next($request);
    }
}
