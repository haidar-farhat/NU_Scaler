<?php
// Simple script to restore the original index.php file

// Check if the backup file exists
if (file_exists(__DIR__ . '/index.php.bak')) {
    // Restore the original file
    if (copy(__DIR__ . '/index.php.bak', __DIR__ . '/index.php')) {
        echo '<h2>Success!</h2>';
        echo '<p>The original index.php file has been restored. The application should now be back online.</p>';
        echo '<p><a href="/">Go to homepage</a></p>';
    } else {
        echo '<h2>Error</h2>';
        echo '<p>Failed to restore the original index.php file. Please check file permissions.</p>';
    }
} else {
    echo '<h2>Error</h2>';
    echo '<p>Backup file not found. Cannot restore the original index.php.</p>';
}
