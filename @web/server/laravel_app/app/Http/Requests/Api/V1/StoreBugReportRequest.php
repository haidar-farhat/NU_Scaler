<?php

namespace App\Http\Requests\Api\V1;

use Illuminate\Foundation\Http\FormRequest;
use Illuminate\Validation\Rule; // Import Rule for In validation

class StoreBugReportRequest extends FormRequest
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
            'description' => 'required|string|max:10000',
            'severity' => ['required', 'string', Rule::in(['low', 'medium', 'high', 'critical'])], // Enum check
            'log_path' => 'nullable|string|max:1024', // Path or identifier for logs
            'system_info' => 'nullable|json', // Expecting a JSON string
            'user_uuid' => 'nullable|uuid|exists:users,uuid',
        ];
    }
}
