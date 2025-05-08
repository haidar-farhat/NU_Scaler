<?php

namespace App\Http\Middleware;

use Closure;
use Illuminate\Http\Request;
use Symfony\Component\HttpFoundation\Response;
use Laravel\Sanctum\PersonalAccessToken;

class ValidateApiToken
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
        $bearer = $request->bearerToken();

        if (!$bearer) {
            return response()->json([
                'message' => 'API token not provided'
            ], Response::HTTP_UNAUTHORIZED);
        }

        $token = PersonalAccessToken::findToken($bearer);

        if (!$token || ($token && !$token->can('api:access'))) {
            return response()->json([
                'message' => 'Invalid or expired API token'
            ], Response::HTTP_UNAUTHORIZED);
        }

        // Token is valid and has required ability
        return $next($request);
    }
}
