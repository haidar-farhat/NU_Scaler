<?php

namespace App\Http\Requests\Api\V1;

use Illuminate\Foundation\Http\FormRequest;
use Illuminate\Validation\Rule;

class StoreHardwareSurveyRequest extends FormRequest
{
    /**
     * Determine if the user is authorized to make this request.
     */
    public function authorize(): bool
    {
        // Anyone can submit a hardware survey
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
            'cpu' => ['nullable', 'string', 'max:255'],
            'gpu' => ['nullable', 'string', 'max:255'],
            'ram_gb' => ['nullable', 'integer', 'min:0'],
            'os' => ['nullable', 'string', 'max:255'],
            'resolution' => ['nullable', 'string', 'max:50', 'regex:/^\d+x\d+$/'], // Validate format like 1920x1080
            // user_uuid is not validated here; it's added in the controller if user is authenticated
        ];
    }
} 