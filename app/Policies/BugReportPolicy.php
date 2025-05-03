<?php

namespace App\Policies;

use App\Models\BugReport;
use App\Models\User;
use Illuminate\Auth\Access\HandlesAuthorization;

class BugReportPolicy
{
    use HandlesAuthorization;

    /**
     * Determine whether the user can view any models.
     *
     * @param  \App\Models\User|null  $user
     * @return bool
     */
    public function viewAny(?User $user): bool
    {
        // Only admin can view lists of bug reports
        return $user && $user->is_admin;
    }

    /**
     * Determine whether the user can view the model.
     *
     * @param  \App\Models\User|null  $user
     * @param  \App\Models\BugReport  $bugReport
     * @return bool
     */
    public function view(?User $user, BugReport $bugReport): bool
    {
        // Admin can view any bug report
        if ($user && $user->is_admin) {
            return true;
        }

        // User can view their own bug report
        if ($user && $bugReport->user_uuid === $user->uuid) {
            return true;
        }

        return false;
    }

    /**
     * Determine whether the user can create models.
     *
     * @param  \App\Models\User|null  $user
     * @return bool
     */
    public function create(?User $user): bool
    {
        // Anyone can submit a bug report, including anonymous users
        return true;
    }

    /**
     * Determine whether the user can update the model.
     *
     * @param  \App\Models\User|null  $user
     * @param  \App\Models\BugReport  $bugReport
     * @return bool
     */
    public function update(?User $user, BugReport $bugReport): bool
    {
        // Only admin can update bug reports
        return $user && $user->is_admin;
    }

    /**
     * Determine whether the user can delete the model.
     *
     * @param  \App\Models\User|null  $user
     * @param  \App\Models\BugReport  $bugReport
     * @return bool
     */
    public function delete(?User $user, BugReport $bugReport): bool
    {
        // Only admin can delete bug reports
        return $user && $user->is_admin;
    }

    /**
     * Determine whether the user can restore the model.
     *
     * @param  \App\Models\User|null  $user
     * @param  \App\Models\BugReport  $bugReport
     * @return bool
     */
    public function restore(?User $user, BugReport $bugReport): bool
    {
        // Only admin can restore bug reports
        return $user && $user->is_admin;
    }

    /**
     * Determine whether the user can permanently delete the model.
     *
     * @param  \App\Models\User|null  $user
     * @param  \App\Models\BugReport  $bugReport
     * @return bool
     */
    public function forceDelete(?User $user, BugReport $bugReport): bool
    {
        // Only admin can force delete bug reports
        return $user && $user->is_admin;
    }
} 