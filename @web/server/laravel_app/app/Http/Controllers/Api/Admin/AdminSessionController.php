<?php

namespace App\Http\Controllers\Api\Admin;

use App\Http\Controllers\Controller;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Log;

class AdminSessionController extends Controller
{
    /**
     * Check if the current session has valid admin auth
     *
     * @param  \Illuminate\Http\Request  $request
     * @return \Illuminate\Http\JsonResponse
     */
    public function check(Request $request)
    {
        $user = $request->user();

        // Log the admin session check
        Log::info('Admin session check', [
            'user_id' => $user ? $user->id : null,
            'email' => $user ? $user->email : null,
            'is_admin' => $user ? ($user->is_admin ?? false) : false,
            'has_token' => $request->bearerToken() ? true : false,
            'token_prefix' => $request->bearerToken() ? substr($request->bearerToken(), 0, 10) . '...' : null,
        ]);

        if (!$user) {
            return response()->json([
                'authenticated' => false,
                'is_admin' => false,
                'message' => 'User not authenticated'
            ], 401);
        }

        if (!$user->is_admin) {
            return response()->json([
                'authenticated' => true,
                'is_admin' => false,
                'message' => 'User is not an admin'
            ], 403);
        }

        // User is authenticated and is an admin
        return response()->json([
            'authenticated' => true,
            'is_admin' => true,
            'user' => [
                'id' => $user->id,
                'name' => $user->name,
                'email' => $user->email,
            ],
            'message' => 'Admin session valid'
        ]);
    }
}
