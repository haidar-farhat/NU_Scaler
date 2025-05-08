<?php

namespace App\Http\Controllers\Api\V1\Auth;

use App\Http\Controllers\Controller;
use Illuminate\Http\Request;
use App\Models\User;
use Illuminate\Support\Facades\Hash;
use Illuminate\Support\Facades\Validator;
use Illuminate\Validation\Rules\Password;
use Illuminate\Http\JsonResponse;

class RegisterController extends Controller
{
    /**
     * Handle a registration request.
     *
     * @param  Request $request
     * @return JsonResponse
     */
    public function register(Request $request): JsonResponse
    {
        $validator = Validator::make($request->all(), [
            'name' => ['required', 'string', 'max:255'],
            'email' => ['required', 'string', 'email', 'max:255', 'unique:users'],
            'password' => ['required', 'string', Password::defaults(), 'confirmed'], // Use default password rules, require confirmation
        ]);

        if ($validator->fails()) {
            return response()->json($validator->errors(), 422);
        }

        $validated = $validator->validated();

        $user = User::create([
            'name' => $validated['name'],
            'email' => $validated['email'],
            'password' => Hash::make($validated['password']),
            // 'is_admin' defaults to false in the migration
            // 'role' is not set here, assuming default or handled differently
        ]);

        // Option 1: Just return success message
        // return response()->json(['message' => 'User registered successfully.'], 201);

        // Option 2: Create a token and return it (log user in automatically)
        $token = $user->createToken('user-registration-token')->plainTextToken;

        return response()->json([
            'message' => 'User registered successfully.',
            'token_type' => 'Bearer',
            'access_token' => $token,
            'user' => [
                 'id' => $user->id,
                 'name' => $user->name,
                 'email' => $user->email,
             ]
        ], 201);

        // TODO: Add welcome email notification
        // $user->notify(new WelcomeNotification());
    }
}
