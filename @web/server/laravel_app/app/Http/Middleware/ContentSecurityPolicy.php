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
                     "script-src 'self' 'unsafe-inline'; " .
                     "style-src 'self' 'unsafe-inline'; " .
                     "img-src 'self' data:; " .
                     "font-src 'self'; " .
                     "connect-src 'self'";

        $response->headers->set('Content-Security-Policy', $cspHeader);

        return $response;
    }
}
