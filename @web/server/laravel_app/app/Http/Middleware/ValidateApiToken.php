<?php

namespace App\Http\Middleware;

use Closure;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Auth;
use Laravel\Sanctum\PersonalAccessToken;
use Symfony\Component\HttpFoundation\Response;

class ValidateApiToken
{
    /**
     * Handle an incoming request.
     *
     * @param  \Illuminate\Http\Request  $request
     * @param  \Closure  $next
     * @param  string|null  $ability
     * @return \Symfony\Component\HttpFoundation\Response
     */
    public function handle(Request $request, Closure $next, ?string $ability = null): Response
    {
        if (!$request->bearerToken()) {
            return response()->json([
                'message' => 'Unauthorized: API token is missing.',
                'status' => 'error',
                'code' => 401,
            ], 401);
        }

        $token = PersonalAccessToken::findToken($request->bearerToken());

        if (!$token || ($ability && !$token->can($ability))) {
            return response()->json([
                'message' => 'Unauthorized: Invalid or expired API token.',
                'status' => 'error',
                'code' => 401,
            ], 401);
        }

        // Check if token is expired (30 days from creation)
        if ($token->created_at->diffInDays(now()) > 30) {
            $token->delete();
            return response()->json([
                'message' => 'Unauthorized: Token has expired.',
                'status' => 'error',
                'code' => 401,
            ], 401);
        }

        // Log token usage
        $token->last_used_at = now();
        $token->save();

        // Set the authenticated user
        Auth::login($token->tokenable);

        return $next($request);
    }
}
