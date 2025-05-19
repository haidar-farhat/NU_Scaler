#!/bin/bash

# Navigate to the Laravel app directory
cd /var/www/NU_Scaler/@web/server/laravel_app

# Create or update .env file with basic settings if it doesn't exist
if [ ! -f .env ]; then
    echo "Creating new .env file..."
    cp .env.example .env 2>/dev/null || cat > .env << EOL
APP_NAME=NuScaler
APP_ENV=production
APP_DEBUG=true
APP_URL=http://localhost
APP_KEY=

LOG_CHANNEL=stack
LOG_LEVEL=debug

DB_CONNECTION=mysql
DB_HOST=db
DB_PORT=3306
DB_DATABASE=nu_scaler
DB_USERNAME=nu_scaler_user
DB_PASSWORD=password

BROADCAST_DRIVER=log
CACHE_DRIVER=file
FILESYSTEM_DISK=local
QUEUE_CONNECTION=sync
SESSION_DRIVER=file
SESSION_LIFETIME=120
EOL
fi

# Generate a new encryption key
NEW_KEY=$(openssl rand -base64 32)

# Update the APP_KEY in the .env file
sed -i "s|APP_KEY=.*|APP_KEY=base64:$NEW_KEY|" .env

# Copy the .env file into the Docker container
docker cp .env nu_scaler-backend-1:/var/www/html/.env

# Clear Laravel config cache
docker exec -i nu_scaler-backend-1 php artisan config:clear
docker exec -i nu_scaler-backend-1 php artisan cache:clear

echo "APP_KEY has been set to: base64:$NEW_KEY"
echo "Configuration has been cleared. The application should now work properly." 