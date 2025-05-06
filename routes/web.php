Route::get('/cors-test', function() {
    return response()->json(['message' => 'CORS test via web routes']);
}); 