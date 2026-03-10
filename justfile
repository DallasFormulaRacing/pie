registry := "ghcr.io/schommie"

# Cross-build both images for arm64
build:
    podman build --platform linux/arm64 -t {{registry}}/ev-can-bridge:latest ./can-bridge
    podman build --platform linux/arm64 -t {{registry}}/ev-gui:latest ./gui

# Push images to GHCR
push:
    podman push {{registry}}/ev-can-bridge:latest
    podman push {{registry}}/ev-gui:latest

# Build and push (run on dev machine)
deploy: build push
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

# Cargo check (cross-compile for raspi)
check:
    cd can-bridge && cargo check
