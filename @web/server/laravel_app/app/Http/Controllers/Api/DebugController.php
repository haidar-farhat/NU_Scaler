<?php

namespace App\Http\Controllers\Api;

use App\Http\Controllers\Controller;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Auth;

class DebugController extends Controller
{
    /**
     * Get debug info about the current authentication state
     *
     * @param  \Illuminate\Http\Request  $request
     * @return \Illuminate\Http\JsonResponse
     */
    public function auth(Request $request)
    {
        $user = $request->user();

        return response()->json([
            'authenticated' => $user !== null,
            'user' => $user ? [
                'id' => $user->id,
                'name' => $user->name,
                'email' => $user->email,
                'is_admin' => $user->is_admin ?? false,
            ] : null,
            'request_has_session' => $request->hasSession(),
            'session_has_auth' => $request->hasSession() ? Auth::guard('web')->check() : false,
            'cookies' => $request->cookies->all(),
            'headers' => [
                'accept' => $request->header('accept'),
                'content-type' => $request->header('content-type'),
                'user-agent' => $request->header('user-agent'),
                'referer' => $request->header('referer'),
                'x-csrf-token' => $request->header('x-csrf-token'),
                'x-xsrf-token' => $request->header('x-xsrf-token'),
            ],
        ]);
    }
}
