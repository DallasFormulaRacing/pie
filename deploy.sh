#!/bin/bash
set -e # Exit on error

echo "Building Backend (ARM64)..."
podman build --platform linux/arm64 -t ghcr.io/schommie/ev-backend:latest ./backend

echo "Building GUI (ARM64)..."
podman build --platform linux/arm64 -t ghcr.io/schommie/ev-gui:latest ./gui

echo "Pushing to GHCR..."
podman push ghcr.io/schommie/ev-backend:latest
podman push ghcr.io/schommie/ev-gui:latest

echo "Done! Run 'sudo podman auto-update' on the Raspi."