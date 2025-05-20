<?php

namespace App\Http\Requests\Api\V1;

use Illuminate\Foundation\Http\FormRequest;

class StoreWebhookRequest extends FormRequest
{
    public function authorize(): bool
    {
        return true;
    }

    public function rules(): array
    {
        return [
            'name' => 'required|string|max:255',
            'url' => 'required|url|max:1000',
            'description' => 'nullable|string|max:1000',
            'events' => 'required|array',
            'events.*' => 'string|in:feedback.review.created,feedback.bug.created,feedback.hardware.created,user.registered',
            'headers' => 'nullable|array',
            'headers.*' => 'string',
        ];
    }
}
