<?php
// Allow from any origin
header("Access-Control-Allow-Origin: *");
header("Access-Control-Allow-Methods: GET, POST, OPTIONS, PUT, DELETE");
header("Access-Control-Allow-Headers: Content-Type, Authorization, X-Requested-With");

// Handle OPTIONS method
if ($_SERVER['REQUEST_METHOD'] === 'OPTIONS') {
    http_response_code(204);
    exit;
}

// Return simple JSON response
echo json_encode([
    'status' => 'success',
    'message' => 'CORS is working!',
    'method' => $_SERVER['REQUEST_METHOD'],
    'timestamp' => date('c')
]);
?> 