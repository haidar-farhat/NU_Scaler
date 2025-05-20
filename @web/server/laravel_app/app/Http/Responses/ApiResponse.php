<?php

namespace App\Http\Responses;

use Illuminate\Http\JsonResponse;

class ApiResponse
{
    public static function success(string $message, $data = null, int $status = 200): JsonResponse
    {
        $response = ['message' => $message];
        if (!is_null($data)) {
            $response['data'] = $data;
        }
        return response()->json($response, $status);
    }

    public static function error(string $message, $errors = null, int $status = 400): JsonResponse
    {
        $response = ['message' => $message];
        if (!is_null($errors)) {
            $response['errors'] = $errors;
        }
        return response()->json($response, $status);
    }

    public static function validation($errors, string $message = 'Validation failed', int $status = 422): JsonResponse
    {
        return response()->json([
            'message' => $message,
            'errors' => $errors,
        ], $status);
    }
}
