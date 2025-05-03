<?php

namespace App\Http\Requests\Api\V1;

use Illuminate\Foundation\Http\FormRequest;
use Illuminate\Validation\Rule;

class StoreBugReportRequest extends FormRequest
{
    /**
     * Determine if the user is authorized to make this request.
     */
    public function authorize(): bool
    {
        // Anyone can submit a bug report
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
            'description' => ['required', 'string', 'max:10000'],
            'log_path' => ['nullable', 'string', 'max:255'], // Assuming path is provided, not file upload yet
            'severity' => ['required', 'string', Rule::in(['low', 'medium', 'high', 'critical'])],
            'system_info' => ['nullable', 'array'],
            'system_info.cpu' => ['nullable', 'string', 'max:255'],
            'system_info.gpu' => ['nullable', 'string', 'max:255'],
            'system_info.ram_gb' => ['nullable', 'integer', 'min:0'],
            'system_info.os' => ['nullable', 'string', 'max:255'],
            // user_uuid is not validated here; it's added in the controller if user is authenticated
        ];
    }
} 