# ALLBRIGHT Dashboard — React + Vite production build for Render
# Multi-stage build: Node.js builder → Nginx runtime

# ============================================================================
# Stage 1: Build
# ============================================================================
FROM node:20-alpine AS builder

# Install build dependencies
RUN apk add --no-cache \
    python3 \
    make \
    g++ \
    git

WORKDIR /app

# Cache npm dependencies - copy package files first
COPY apps/dashboard/package*.json ./

# Install dependencies (including devDependencies for build)
RUN npm ci

# Copy source code
COPY apps/dashboard/ .

# Build the React app for production
# This creates the dist/ folder with static assets
RUN npm run build

# ============================================================================
# Stage 2: Production Runtime (Nginx)
# ============================================================================
FROM nginx:alpine

# Install curl for health checks
RUN apk add --no-cache curl

# Copy built assets from builder stage
COPY --from=builder /app/dist /usr/share/nginx/html

# Copy nginx configuration
COPY apps/dashboard/nginx.conf /etc/nginx/conf.d/default.conf

# Create health check endpoint
RUN echo "healthy" > /usr/share/nginx/html/healthz

# Expose port 80 (Render's default web port)
EXPOSE 80

# Add health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost/healthz || exit 1

# Start nginx in foreground
CMD ["nginx", "-g", "daemon off;"]
