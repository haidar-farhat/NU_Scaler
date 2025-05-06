<?php

namespace App\Http\Controllers;

use Illuminate\Http\Request;

class TestController extends Controller
{
    public function testCors()
    {
        return response()->json([
            'message' => 'CORS test successful',
            'timestamp' => now()->toIso8601String()
        ]);
    }
} 