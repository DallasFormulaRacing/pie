registry := "ghcr.io/schommie"

# Cross-build both images for arm64
build:
    just --parallel build-backend build-gui

build-backend:
    podman build --platform linux/arm64 -t {{registry}}/ev-can-bridge:latest ./backend_v3/backend

build-gui:
    podman build --platform linux/arm64 -t {{registry}}/ev-gui:latest ./gui

# Push images to GHCR
push:
    just --parallel push-backend push-gui

push-backend:
    podman push {{registry}}/ev-can-bridge:latest

push-gui:
    podman push {{registry}}/ev-gui:latest

# Build and push (run on dev machine)
deploy:
    just --parallel build-backend build-gui
    just --parallel push-backend push-gui
    @echo "Done. Run 'sudo podman auto-update' on the Raspi."

# Start dev environment
dev:
    podman-compose -f compose.dev.yml up --build

# Start prod stack
up:
    podman-compose up -d

# Stop all services
down:
    podman-compose down
