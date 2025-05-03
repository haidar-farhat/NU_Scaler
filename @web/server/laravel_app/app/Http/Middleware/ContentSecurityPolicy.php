<?php

namespace App\Http\Middleware;

use Closure;
use Illuminate\Http\Request;
use Symfony\Component\HttpFoundation\Response;

class ContentSecurityPolicy
{
    /**
     * Handle an incoming request.
     *
     * @param  \Illuminate\Http\Request  $request
     * @param  \Closure  $next
     * @return \Symfony\Component\HttpFoundation\Response
     */
    public function handle(Request $request, Closure $next): Response
    {
        $response = $next($request);

        // Set Content Security Policy header
        $cspHeader = "default-src 'self'; " .
                     "script-src 'self' 'unsafe-inline' https://cdn.jsdelivr.net https://*.google-analytics.com; " .
                     "style-src 'self' 'unsafe-inline' https://cdn.jsdelivr.net https://fonts.googleapis.com; " .
                     "img-src 'self' data: https://*.google-analytics.com; " .
                     "font-src 'self' https://fonts.gstatic.com; " .
                     "connect-src 'self' https://*.google-analytics.com https://nu-scaler.com; " .
                     "frame-src 'none'; " .
                     "object-src 'none'; " .
                     "base-uri 'self'; " .
                     "form-action 'self'; " .
                     "frame-ancestors 'none';";

        $response->headers->set('Content-Security-Policy', $cspHeader);

        return $response;
    }
}
