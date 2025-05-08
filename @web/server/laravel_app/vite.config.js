import { defineConfig } from 'vite';
import laravel from 'laravel-vite-plugin';
import tailwindcss from '@tailwindcss/vite';

export default defineConfig({
    plugins: [
        laravel({
            input: ['resources/css/app.css', 'resources/js/app.js'],
            refresh: true,
        }),
        tailwindcss(),
    ],
    build: {
        sourcemap: true
    },
    server: {
        hmr: {
            host: 'localhost'
        },
        // Prevent React DevTools source map errors
        fs: {
            strict: false
        }
    }
});
