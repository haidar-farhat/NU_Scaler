FROM php:8.1-fpm

# Install system dependencies
RUN apt-get update && apt-get install -y \
    git \
    unzip \
    libpng-dev \
    libjpeg62-turbo-dev \
    libfreetype6-dev \
    libonig-dev \
    libxml2-dev \
    zip \
    curl \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

# Install PHP extensions
RUN docker-php-ext-configure gd --with-freetype --with-jpeg \
    && docker-php-ext-install pdo_mysql mbstring exif pcntl bcmath gd xml

# Get latest Composer
COPY --from=composer:latest /usr/bin/composer /usr/bin/composer

# Create system user to run Composer and Artisan Commands
RUN useradd -G www-data,root -u 1000 -d /home/sail sail
RUN mkdir -p /home/sail/.composer && \
    chown -R sail:sail /home/sail

WORKDIR /var/www/html

# Copy existing application directory contents (permissions handled later)
COPY . /var/www/html

# Change current user to sail
USER sail

# Install composer dependencies
RUN composer install --ignore-platform-reqs --no-interaction --no-plugins --no-scripts --prefer-dist --optimize-autoloader

# Copy back to host to ensure ownership is correct after install
COPY --chown=sail:sail . /var/www/html

# Switch back to root user
USER root

# Clear caches and optimize (as root to ensure permissions)
# RUN php artisan cache:clear && php artisan config:clear && php artisan route:clear && php artisan view:clear
# Optimization commands are better run after composer install in the build or manually inside container

# Set permissions for storage and bootstrap cache
RUN chown -R www-data:www-data /var/www/html/storage /var/www/html/bootstrap/cache

# Expose port 9000 and start php-fpm server
EXPOSE 9000
CMD ["php-fpm"]
