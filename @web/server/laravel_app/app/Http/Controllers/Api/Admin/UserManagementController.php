<?php

namespace App\Http\Controllers\Api\Admin;

use App\Http\Controllers\Controller;
use App\Models\User;
use Illuminate\Http\Request;
use Illuminate\Validation\Rule;

class UserManagementController extends Controller
{
    // List users (paginated)
    public function index(Request $request)
    {
        $users = User::query()
            ->select('id', 'name', 'email', 'is_admin', 'is_active', 'created_at', 'updated_at')
            ->orderByDesc('created_at')
            ->paginate($request->get('per_page', 20));
        return response()->json($users);
    }

    // Promote/demote user
    public function updateRole(Request $request, User $user)
    {
        $request->validate([
            'is_admin' => ['required', 'boolean'],
        ]);
        // Prevent self-demotion
        if ($user->id === $request->user()->id) {
            return response()->json(['message' => 'You cannot change your own admin status.'], 403);
        }
        $user->is_admin = $request->is_admin;
        $user->save();
        return response()->json(['message' => 'User role updated.', 'user' => $user]);
    }

    // Activate/deactivate user
    public function updateStatus(Request $request, User $user)
    {
        $request->validate([
            'is_active' => ['required', 'boolean'],
        ]);
        // Prevent self-deactivation
        if ($user->id === $request->user()->id) {
            return response()->json(['message' => 'You cannot change your own active status.'], 403);
        }
        $user->is_active = $request->is_active;
        $user->save();
        return response()->json(['message' => 'User status updated.', 'user' => $user]);
    }
}
