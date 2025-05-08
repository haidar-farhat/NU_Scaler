<?php

namespace App\Http\Middleware;

use Closure;
use Illuminate\Http\Request;
use Symfony\Component\HttpFoundation\Response;
use Illuminate\Support\Facades\Log;

class IsAdmin
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
        $user = $request->user();

        // Log authentication attempt for debugging
        Log::info('Admin check', [
            'user' => $user ? [
                'id' => $user->id,
                'email' => $user->email,
                'is_admin' => $user->is_admin ?? false
            ] : null,
            'authenticated' => $user !== null,
            'has_bearer_token' => $request->bearerToken() !== null,
            'request_has_session' => $request->hasSession(),
        ]);

        if (!$user) {
            return response()->json(['message' => 'Unauthorized. Not authenticated.'], Response::HTTP_UNAUTHORIZED);
        }

        if (!$user->isAdmin()) {
            return response()->json(['message' => 'Forbidden. Admin access required.'], Response::HTTP_FORBIDDEN);
        }

        return $next($request);
    }
}
