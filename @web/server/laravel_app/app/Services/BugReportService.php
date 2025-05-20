<?php

namespace App\Services;

use App\Models\BugReport;

class BugReportService
{
    public function create(array $data): BugReport
    {
        return BugReport::create($data);
    }
}
