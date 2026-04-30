registry := "ghcr.io/schommie"

# Cross-build both images for arm64
build:
    @bash -u -o pipefail -c 'status=0; just build-backend & backend=$!; just build-gui & gui=$!; wait $backend || status=$?; wait $gui || status=$?; exit $status'

build-backend:
    podman build --platform linux/arm64 -t {{registry}}/ev-can-bridge:latest ./backend_v3/backend

build-gui:
    podman build --platform linux/arm64 -t {{registry}}/ev-gui:latest ./gui

# Push images to GHCR
push:
    @bash -u -o pipefail -c 'status=0; just push-backend & backend=$!; just push-gui & gui=$!; wait $backend || status=$?; wait $gui || status=$?; exit $status'

push-backend:
    podman push {{registry}}/ev-can-bridge:latest

push-gui:
    podman push {{registry}}/ev-gui:latest

# Build and push (run on dev machine)
deploy:
    @bash -u -o pipefail -c 'status=0; just build-backend & backend=$!; just build-gui & gui=$!; wait $backend || status=$?; wait $gui || status=$?; exit $status'
    @bash -u -o pipefail -c 'status=0; just push-backend & backend=$!; just push-gui & gui=$!; wait $backend || status=$?; wait $gui || status=$?; exit $status'
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

login:
    sudo podman login {{registry}} --username schommie --password-stdin < ~/.ghtoken
