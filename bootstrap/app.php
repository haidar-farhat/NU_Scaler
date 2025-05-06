<?php

use Illuminate\Support\Facades\Route;
use Illuminate\Support\Facades\App;
use Illuminate\Support\Facades\Config;
use Illuminate\Support\Facades\Log;
use Illuminate\Support\Facades\Request;
use Illuminate\Support\Facades\Response;
use Illuminate\Support\Facades\Session;
use Illuminate\Support\Facades\View;
use Illuminate\Support\Facades\Http;
use Illuminate\Support\Facades\File;
use Illuminate\Support\Facades\Storage;
use Illuminate\Support\Facades\Cache;
use Illuminate\Support\Facades\Queue;
use Illuminate\Support\Facades\Broadcast;
use Illuminate\Support\Facades\Auth;
use Illuminate\Support\Facades\Passport;
use Illuminate\Support\Facades\JWT;
use Illuminate\Support\Facades\JWTAuth;
use Illuminate\Support\Facades\JWTFactory;
use Illuminate\Support\Facades\JWK;
use Illuminate\Support\Facades\JWKS;
use Illuminate\Support\Facades\JWKSBuilder;
use Illuminate\Support\Facades\JWKSToPEM;
use Illuminate\Support\Facades\JWKSToPEMBuilder;
use Illuminate\Support\Facades\JWKSToPEMConverter;
use Illuminate\Support\Facades\JWKSToPEMConverterBuilder;
use Illuminate\Support\Facades\JWKSToPEMConverterBuilderBuilder;
use Illuminate\Support\Facades\JWKSToPEMConverterBuilderBuilderBuilder;
use Illuminate\Support\Facades\JWKSToPEMConverterBuilderBuilderBuilderBuilder;
use Illuminate\Support\Facades\JWKSToPEMConverterBuilderBuilderBuilderBuilderBuild;
use Illuminate\Support\Facades\JWKSToPEMConverterBuilderBuilderBuilderBuilderBuildBuild;
use Illuminate\Support\Facades\JWKSToPEMConverterBuilderBuilderBuilderBuilderBuildBuildBuild;
use Illuminate\Support\Facades\JWKSToPEMConverterBuilderBuilderBuilderBuilderBuildBuildBuildBuild;
use Illuminate\Support\Facades\JWKSToPEMConverterBuilderBuilderBuilderBuilderBuildBuildBuildBuildBuild;
use Illuminate\Support\Facades\JWKSToPEMConverterBuilderBuilderBuilderBuilderBuildBuildBuildBuildBuildBuild;
use Illuminate\Support\Facades\JWKSToPEMConverterBuilderBuilderBuilderBuilderBuildBuildBuildBuildBuildBuildBuild;
use Illuminate\Support\Facades\JWKSToPEMConverterBuilderBuilderBuilderBuilderBuildBuildBuildBuildBuildBuildBuildBuild;
use Illuminate\Support\Facades\JWKSToPEMConverterBuilderBuilderBuilderBuilderBuildBuildBuildBuildBuildBuildBuildBuildBuild;
use Illuminate\Support\Facades\JWKSToPEMConverterBuilderBuilderBuilderBuilderBuildBuildBuildBuildBuildBuildBuildBuildBuild;
use Illuminate\Support\Facades\JWKSToPEMConverterBuilderBuilderBuilderBuilderBuildBuildBuildBuildBuildBuildBuildBuildBuild;
use Illuminate\Support\Facades\JWKSToPEMConverterBuilderBuilderBuilderBuilderBuildBuildBuildBuildBuildBuildBuildBuildBuild;
use Illuminate\Support\Facades\JWKSToPEMConverterBuilderBuilderBuilderBuilderBuildBuildBuildBuildBuildBuildBuildBuildBuild;
use Illuminate\Support\Facades\JWKSToPEMConverterBuilderBuilderBuilderBuilderBuildBuildBuildBuildBuildBuildBuildBuildBuild;
use Illuminate\Support\Facades\JWKSToPEMConverterBuilderBuilderBuilderBuilderBuildBuildBuildBuildBuildBuildBuildBuildBuild;
use Illuminate\Support\Facades\JWKSToPEMConverterBuilderBuilderBuilderBuilderBuildBuildBuildBuildBuildBuildBuildBuildBuild;
use Illuminate\Support\Facades\JWKSToPEMConverterBuilderBuilderBuilderBuilderBuildBuildBuildBuildBuildBuildBuildBuildBuild;
use Illuminate\Support\Facades\JWKSToPEMConverterBuilderBuilderBuilderBuilderBuildBuildBuildBuildBuildBuildBuildBuildBuild;
use Illuminate\Support\Facades\JWKSToPEMConverterBuilderBuilderBuilderBuilderBuildBuildBuildBuildBuildBuildBuildBuildBuild;
use Illuminate\Support\Facades\JWKSToPEMConverterBuilderBuilderBuilderBuilderBuildBuildBuildBuildBuildBuildBuildBuildBuild;

$app->instance('path.public', base_path('public'));

$app->withoutMiddleware([
    \App\Http\Middleware\ApiRateLimiter::class,
]);

$app->afterResolving(\Illuminate\Contracts\Debug\ExceptionHandler::class, function ($handler) {
    $handler->renderable(function (\Throwable $e, $request) {
        $response = response()->json([
            'error' => 'Server Error',
            'message' => $e->getMessage(),
            'trace' => $e->getTraceAsString(),
        ], 500);
        
        // Force CORS headers on all responses, even errors
        return $response->withHeaders([
            'Access-Control-Allow-Origin' => '*',
            'Access-Control-Allow-Methods' => 'GET, POST, PUT, DELETE, OPTIONS',
            'Access-Control-Allow-Headers' => 'Content-Type, Authorization',
        ]);
    });
});

return $app;
