<?php

namespace App\Services;

use App\Models\User;
use App\Repositories\UserRepository;
use Illuminate\Support\Facades\Hash;
use Illuminate\Support\Facades\Log;

class UserService
{
    protected $userRepository;

    public function __construct(UserRepository $userRepository)
    {
        $this->userRepository = $userRepository;
    }

    public function register(array $data): User
    {
        $data['password'] = Hash::make($data['password']);
        return $this->userRepository->create($data);
    }

    public function login(string $email, string $password): ?User
    {
        $user = $this->userRepository->findByEmail($email);
        if ($user && Hash::check($password, $user->password)) {
            Log::info('User login successful', [
                'user_id' => $user->id,
                'email' => $user->email,
                'is_admin' => $user->is_admin ?? false,
            ]);
            return $user;
        }
        return null;
    }

    public function update(User $user, array $data): User
    {
        return $this->userRepository->update($user, $data);
    }
}
