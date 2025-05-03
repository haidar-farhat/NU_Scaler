<?php

namespace App\Http\Requests;

use Illuminate\Foundation\Http\FormRequest;

class HardwareSurveyRequest extends FormRequest
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
            'cpu_model' => 'required|string|max:255',
            'gpu_model' => 'required|string|max:255',
            'ram_size' => 'required|integer|min:1',
            'os' => 'required|string|max:255',
            'resolution' => 'required|string|max:50',
            'monitor_refresh_rate' => 'nullable|integer|min:1|max:360',
            'additional_info' => 'nullable|string|max:1000',
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
            'cpu_model.required' => 'CPU model information is required',
            'gpu_model.required' => 'GPU model information is required',
            'ram_size.required' => 'RAM size is required',
            'ram_size.integer' => 'RAM size must be a number',
            'ram_size.min' => 'RAM size must be at least 1GB',
            'os.required' => 'Operating system information is required',
            'resolution.required' => 'Screen resolution information is required',
            'monitor_refresh_rate.integer' => 'Refresh rate must be a number',
            'monitor_refresh_rate.min' => 'Refresh rate must be at least 1Hz',
            'monitor_refresh_rate.max' => 'Refresh rate cannot be more than 360Hz',
        ];
    }
}
