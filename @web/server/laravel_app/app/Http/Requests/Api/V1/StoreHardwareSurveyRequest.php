<?php

namespace App\Http\Requests\Api\V1;

use Illuminate\Foundation\Http\FormRequest;

class StoreHardwareSurveyRequest extends FormRequest
{
    /**
     * Determine if the user is authorized to make this request.
     *
     * @return bool
     */
    public function authorize(): bool
    {
        // Public endpoint
        return true;
    }

    /**
     * Get the validation rules that apply to the request.
     *
     * @return array<string, \Illuminate\Contracts\Validation\ValidationRule|array<mixed>|string>
     */
    public function rules(): array
    {
        return [
            'gpu' => 'required|string|max:255', // Making GPU required for usefulness
            'cpu' => 'nullable|string|max:255',
            'ram_gb' => 'nullable|integer|min:1',
            'os' => 'nullable|string|max:255',
            'resolution' => 'nullable|string|max:50', // e.g., 1920x1080
            'user_uuid' => 'nullable|uuid|exists:users,uuid',
        ];
    }
}
