<?php
// Create a backup of the original index.php if it doesn't exist yet
if (!file_exists(__DIR__ . '/index.php.bak')) {
    copy(__FILE__, __DIR__ . '/index.php.bak');
}

// Output maintenance page instead of running Laravel application
header('HTTP/1.1 503 Service Temporarily Unavailable');
header('Content-Type: text/html; charset=UTF-8');
?>
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>NU_Scaler - Temporarily Unavailable</title>
    <link rel="preconnect" href="https://fonts.bunny.net">
    <link href="https://fonts.bunny.net/css?family=instrument-sans:400,500,600" rel="stylesheet" />
    <style>
        body {
            font-family: 'Instrument Sans', sans-serif;
            background-color: #FDFDFC;
            color: #1b1b18;
            padding: 2rem;
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            min-height: 100vh;
            margin: 0;
            line-height: 1.5;
        }
        .container {
            max-width: 600px;
            width: 100%;
            background-color: white;
            border-radius: 0.5rem;
            box-shadow: inset 0px 0px 0px 1px rgba(26,26,0,0.16);
            padding: 2.5rem;
        }
        .header {
            display: flex;
            align-items: center;
            margin-bottom: 1.5rem;
        }
        .logo {
            height: 48px;
            margin-right: 1rem;
            color: #F53003;
        }
        h1 {
            font-weight: 500;
            font-size: 1.5rem;
            margin: 0 0 1rem 0;
        }
        h2 {
            font-weight: 500;
            font-size: 1.2rem;
            margin: 1.5rem 0 0.5rem 0;
        }
        p {
            margin: 0 0 1rem 0;
            color: #706f6c;
        }
        ul {
            padding-left: 1.5rem;
            margin: 0.5rem 0 1rem 0;
        }
        li {
            margin-bottom: 0.5rem;
            color: #706f6c;
        }
        .status-badge {
            display: inline-block;
            background-color: #fff2f2;
            border-radius: 0.25rem;
            padding: 0.25rem 0.75rem;
            font-weight: 500;
            color: #F53003;
            margin-bottom: 1rem;
        }
        @media (prefers-color-scheme: dark) {
            body {
                background-color: #0a0a0a;
                color: #EDEDEC;
            }
            .container {
                background-color: #161615;
                box-shadow: inset 0px 0px 0px 1px rgba(255,250,237,0.18);
            }
            p, li {
                color: #A1A09A;
            }
            .status-badge {
                background-color: #1D0002;
                color: #FF4433;
            }
            .logo {
                color: #F61500;
            }
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <svg class="logo" viewBox="0 0 50 50" fill="none" xmlns="http://www.w3.org/2000/svg">
                <path d="M2 0H0V30H10V25H2V0Z" fill="currentColor" />
                <path d="M35 12.5C36.5 15 37.5 18 37.5 21C37.5 24 36.5 27 35 29.5C33.5 32 31.5 34 29 35.5C26.5 37 23.5 38 20.5 38C17.5 38 14.5 37 12 35.5C9.5 34 7.5 32 6 29.5C4.5 27 3.5 24 3.5 21C3.5 18 4.5 15 6 12.5C7.5 10 9.5 8 12 6.5C14.5 5 17.5 4 20.5 4C23.5 4 26.5 5 29 6.5C31.5 8 33.5 10 35 12.5Z" fill="currentColor" />
                <path d="M50 0H48V30H50V0Z" fill="currentColor" />
            </svg>
            <h1>NU_Scaler</h1>
        </div>

        <div class="status-badge">Application Unavailable</div>

        <p>The application is currently unavailable due to dependency issues. Please try again later.</p>

        <h2>Troubleshooting:</h2>
        <ul>
            <li>Check internet connection</li>
            <li>Try changing npm registry (npm config set registry https://registry.npmjs.org/)</li>
            <li>Run npm install with --force or --legacy-peer-deps flag</li>
        </ul>

        <p>Our team has been notified and is working to resolve this issue as quickly as possible. We apologize for any inconvenience.</p>
    </div>
</body>
</html>
