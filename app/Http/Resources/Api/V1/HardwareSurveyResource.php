<?php

namespace App\Http\Resources\Api\V1;

use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;

class HardwareSurveyResource extends JsonResource
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
            'cpu' => $this->cpu,
            'gpu' => $this->gpu,
            'ram' => $this->ram,
            'os' => $this->os,
            'resolution' => $this->resolution,
            'user_uuid' => $this->when($request->user() && $request->user()->is_admin, $this->user_uuid),
            'created_at' => $this->created_at->toIso8601String(),
            'updated_at' => $this->updated_at->toIso8601String(),
        ];
    }
} 