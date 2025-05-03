<?php

namespace App\Policies;

use App\Models\HardwareSurvey;
use App\Models\User;
use Illuminate\Auth\Access\HandlesAuthorization;

class HardwareSurveyPolicy
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
        // Only admin can view lists of hardware surveys
        return $user && $user->is_admin;
    }

    /**
     * Determine whether the user can view the model.
     *
     * @param  \App\Models\User|null  $user
     * @param  \App\Models\HardwareSurvey  $hardwareSurvey
     * @return bool
     */
    public function view(?User $user, HardwareSurvey $hardwareSurvey): bool
    {
        // Admin can view any hardware survey
        if ($user && $user->is_admin) {
            return true;
        }

        // User can view their own hardware survey
        if ($user && $hardwareSurvey->user_uuid === $user->uuid) {
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
        // Anyone can submit a hardware survey, including anonymous users
        return true;
    }

    /**
     * Determine whether the user can update the model.
     *
     * @param  \App\Models\User|null  $user
     * @param  \App\Models\HardwareSurvey  $hardwareSurvey
     * @return bool
     */
    public function update(?User $user, HardwareSurvey $hardwareSurvey): bool
    {
        // Admin can update any hardware survey
        if ($user && $user->is_admin) {
            return true;
        }

        // Users cannot update their hardware surveys after submission
        return false;
    }

    /**
     * Determine whether the user can delete the model.
     *
     * @param  \App\Models\User|null  $user
     * @param  \App\Models\HardwareSurvey  $hardwareSurvey
     * @return bool
     */
    public function delete(?User $user, HardwareSurvey $hardwareSurvey): bool
    {
        // Only admin can delete hardware surveys
        return $user && $user->is_admin;
    }

    /**
     * Determine whether the user can restore the model.
     *
     * @param  \App\Models\User|null  $user
     * @param  \App\Models\HardwareSurvey  $hardwareSurvey
     * @return bool
     */
    public function restore(?User $user, HardwareSurvey $hardwareSurvey): bool
    {
        // Only admin can restore hardware surveys
        return $user && $user->is_admin;
    }

    /**
     * Determine whether the user can permanently delete the model.
     *
     * @param  \App\Models\User|null  $user
     * @param  \App\Models\HardwareSurvey  $hardwareSurvey
     * @return bool
     */
    public function forceDelete(?User $user, HardwareSurvey $hardwareSurvey): bool
    {
        // Only admin can force delete hardware surveys
        return $user && $user->is_admin;
    }
} 