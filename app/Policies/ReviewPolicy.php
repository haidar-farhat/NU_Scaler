<?php

namespace App\Policies;

use App\Models\Review;
use App\Models\User;
use Illuminate\Auth\Access\HandlesAuthorization;

class ReviewPolicy
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
        // Admin can view all reviews
        if ($user && $user->is_admin) {
            return true;
        }

        // Public can view approved reviews or their own
        return false;
    }

    /**
     * Determine whether the user can view the model.
     *
     * @param  \App\Models\User|null  $user
     * @param  \App\Models\Review  $review
     * @return bool
     */
    public function view(?User $user, Review $review): bool
    {
        // Admin can view any review
        if ($user && $user->is_admin) {
            return true;
        }

        // User can view their own review
        if ($user && $review->user_uuid === $user->uuid) {
            return true;
        }

        // Public can't view individual reviews
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
        // Anyone can create a review, even anonymous
        return true;
    }

    /**
     * Determine whether the user can update the model.
     *
     * @param  \App\Models\User|null  $user
     * @param  \App\Models\Review  $review
     * @return bool
     */
    public function update(?User $user, Review $review): bool
    {
        // Admin can update any review
        if ($user && $user->is_admin) {
            return true;
        }

        // User can update their own review if it's recent (within 24 hours)
        if ($user && $review->user_uuid === $user->uuid) {
            $dayAgo = now()->subDay();
            return $review->created_at->gt($dayAgo);
        }

        return false;
    }

    /**
     * Determine whether the user can delete the model.
     *
     * @param  \App\Models\User|null  $user
     * @param  \App\Models\Review  $review
     * @return bool
     */
    public function delete(?User $user, Review $review): bool
    {
        // Only admin can delete reviews
        return $user && $user->is_admin;
    }

    /**
     * Determine whether the user can restore the model.
     *
     * @param  \App\Models\User|null  $user
     * @param  \App\Models\Review  $review
     * @return bool
     */
    public function restore(?User $user, Review $review): bool
    {
        // Only admin can restore reviews
        return $user && $user->is_admin;
    }

    /**
     * Determine whether the user can permanently delete the model.
     *
     * @param  \App\Models\User|null  $user
     * @param  \App\Models\Review  $review
     * @return bool
     */
    public function forceDelete(?User $user, Review $review): bool
    {
        // Only admin can force delete reviews
        return $user && $user->is_admin;
    }
} 