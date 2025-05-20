<?php

namespace App\Services;

use App\Models\HardwareSurvey;

class HardwareSurveyService
{
    public function create(array $data): HardwareSurvey
    {
        return HardwareSurvey::create($data);
    }
}
