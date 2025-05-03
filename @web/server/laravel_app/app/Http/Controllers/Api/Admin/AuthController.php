<?php

namespace App\Http\Controllers\Api\Admin;

use App\Http\Controllers\Controller;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Auth;
use Illuminate\Validation\ValidationException;
use App\Models\User; // Import User model
use Illuminate\Http\JsonResponse;

class AuthController extends Controller
{
    /**
     * Handle an admin authentication attempt.
     *
     * @param  \Illuminate\Http\Request  $request
     * @return \Illuminate\Http\JsonResponse
     */
    public function login(Request $request): JsonResponse
    {
        $request->validate([
            'email' => 'required|email',
            'password' => 'required|string',
        ]);

        // Attempt authentication with credentials
        if (!Auth::attempt($request->only('email', 'password'))) {
            throw ValidationException::withMessages([
                'email' => [__('auth.failed')], // Use standard localization key
            ]);
        }

        // Check if the authenticated user is an admin
        $user = Auth::user();
        if (!$user instanceof User || !$user->is_admin) {
            // Log out the non-admin user who managed to authenticate
            Auth::logout();

            // Return a forbidden error - don't reveal if the user exists but isn't admin
            return response()->json(['message' => __('auth.failed')], 403);
        }

        // Credentials are valid, user is admin - Generate Sanctum token
        // Consider adding a token name, e.g., 'admin-login'
        $token = $user->createToken('admin-api-token')->plainTextToken;

        return response()->json([
            'message' => 'Admin login successful.',
            'token_type' => 'Bearer',
            'access_token' => $token,
            'user' => [
                'id' => $user->id,
                'name' => $user->name,
                'email' => $user->email,
            ] // Return basic user info
        ]);
    }

    /**
     * Log the admin user out (Invalidate the token).
     *
     * @param  \Illuminate\Http\Request  $request
     * @return \Illuminate\Http\JsonResponse
     */
    public function logout(Request $request): JsonResponse
    {
        $request->user()->currentAccessToken()->delete();

        return response()->json(['message' => 'Admin logged out successfully.']);
    }
}
