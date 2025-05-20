<?php

namespace App\Http\Controllers\Api\Admin;

use App\Http\Controllers\Controller;
use App\Models\User;
use Illuminate\Http\Request;
use Illuminate\Validation\Rule;
use App\Http\Responses\ApiResponse;
use App\Services\UserService;

class UserManagementController extends Controller
{
    protected $userService;

    public function __construct(UserService $userService)
    {
        $this->userService = $userService;
    }

    // List users (paginated)
    public function index(Request $request)
    {
        $users = User::query()
            ->select('id', 'name', 'email', 'is_admin', 'is_active', 'created_at', 'updated_at')
            ->orderByDesc('created_at')
            ->paginate($request->get('per_page', 20));
        return ApiResponse::success('Users fetched successfully', $users);
    }

    // Promote/demote user
    public function updateRole(Request $request, User $user)
    {
        $request->validate([
            'is_admin' => ['required', 'boolean'],
        ]);
        // Prevent self-demotion
        if ($user->id === $request->user()->id) {
            return ApiResponse::error('You cannot change your own admin status.', null, 403);
        }
        $user = $this->userService->update($user, ['is_admin' => $request->is_admin]);
        return ApiResponse::success('User role updated.', $user);
    }

    // Activate/deactivate user
    public function updateStatus(Request $request, User $user)
    {
        $request->validate([
            'is_active' => ['required', 'boolean'],
        ]);
        // Prevent self-deactivation
        if ($user->id === $request->user()->id) {
            return ApiResponse::error('You cannot change your own active status.', null, 403);
        }
        $user = $this->userService->update($user, ['is_active' => $request->is_active]);
        return ApiResponse::success('User status updated.', $user);
    }
}
