<?php

namespace App\Http\Requests;

use Illuminate\Foundation\Http\FormRequest;

class BugReportRequest extends FormRequest
{
    /**
     * Determine if the user is authorized to make this request.
     */
    public function authorize(): bool
    {
        return true; // Public endpoint, anyone can submit
    }

    /**
     * Get the validation rules that apply to the request.
     *
     * @return array<string, \Illuminate\Contracts\Validation\ValidationRule|array<mixed>|string>
     */
    public function rules(): array
    {
        return [
            'description' => 'required|string|max:2000',
            'category' => 'required|string|in:ui,performance,feature,crash,other',
            'severity' => 'required|string|in:low,medium,high,critical',
            'steps_to_reproduce' => 'nullable|string|max:2000',
            'system_info' => 'required|array',
            'system_info.os' => 'required|string|max:255',
            'system_info.browser' => 'nullable|string|max:255',
            'system_info.device' => 'nullable|string|max:255',
            'system_info.app_version' => 'required|string|max:50',
        ];
    }

    /**
     * Get custom messages for validator errors.
     *
     * @return array<string, string>
     */
    public function messages(): array
    {
        return [
            'description.required' => 'A description of the bug is required',
            'category.required' => 'Please select a category for this bug',
            'category.in' => 'Please select a valid category',
            'severity.required' => 'Please select a severity level',
            'severity.in' => 'Please select a valid severity level',
            'system_info.required' => 'System information is required',
            'system_info.os.required' => 'Operating system information is required',
            'system_info.app_version.required' => 'Application version is required',
        ];
    }
}
