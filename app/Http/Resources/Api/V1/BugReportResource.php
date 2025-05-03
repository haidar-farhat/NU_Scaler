<?php

namespace App\Http\Resources\Api\V1;

use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;

class BugReportResource extends JsonResource
{
    /**
     * Transform the resource into an array.
     *
     * @return array<string, mixed>
     */
    public function toArray(Request $request): array
    {
        return [
            'id' => $this->id,
            'severity' => $this->severity,
            'description' => $this->description,
            'steps_to_reproduce' => $this->steps_to_reproduce,
            'log_path' => $this->when($request->user() && $request->user()->is_admin, $this->log_path),
            'user_uuid' => $this->when($request->user() && $request->user()->is_admin, $this->user_uuid),
            'created_at' => $this->created_at->toIso8601String(),
            'updated_at' => $this->updated_at->toIso8601String(),
        ];
    }
} 