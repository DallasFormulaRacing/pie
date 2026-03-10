# ev-26-raspi

CAN bridge + web GUI for the DFR EV-26 FSAE car. Runs on a Raspberry Pi connected to the vehicle CAN bus.

- **can-bridge** — Rust backend that reads/writes CAN FD frames and exposes a WebSocket API (port 9002)
- **gui** — React frontend served by Caddy, reverse proxies `/ws` to the backend

## Prerequisites

- [Podman](https://podman.io/) (or Docker)
- [just](https://github.com/casey/just) command runner
- GHCR login: `podman login ghcr.io`

## Just Commands

| Command | Description |
|---------|-------------|
| `just build` | Cross-build both container images for arm64 |
| `just push` | Push images to `ghcr.io/schommie/` |
| `just deploy` | Build + push, then run `podman auto-update` on the Pi |
| `just dev` | Start dev environment with hot reload |
| `just up` | Start production stack on the Pi |
| `just down` | Stop all services |
| `just check` | Run `cargo check` for can-bridge (cross-compile) |

## CI/CD Workflow

### Deploy from dev machine

```sh
just deploy
```

This builds arm64 images, pushes them to GHCR, and prints a reminder to update the Pi.

### On the Raspberry Pi

Pull and restart with the latest images:

```sh
sudo podman auto-update
```

Or manually:

```sh
podman-compose pull && podman-compose up -d
```

### Auto-start on boot

Place Quadlet `.container` files in `/etc/containers/systemd/` on the Pi:

**`/etc/containers/systemd/can-bridge.container`**
```ini
[Container]
Image=ghcr.io/schommie/ev-can-bridge:latest
Network=host
AutoUpdate=registry

[Service]
Restart=always

[Install]
WantedBy=default.target
```

**`/etc/containers/systemd/gui.container`**
```ini
[Container]
Image=ghcr.io/schommie/ev-gui:latest
Network=host
AutoUpdate=registry

[Service]
Restart=always

[Install]
WantedBy=default.target
```

Then reload and start:

```sh
sudo systemctl daemon-reload
sudo systemctl start can-bridge gui
```

## Local Development

```sh
just dev
```

Starts both services with volume mounts for hot reload — can-bridge uses `cargo-watch`, gui uses Vite dev server on port 5173.
