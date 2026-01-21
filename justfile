# Ember Network Connect - Development Commands
# Run `just --list` to see available commands

# Default recipe - show help
default:
    @just --list

# Build UI (requires Node.js and pnpm)
build-ui:
    cd ui && pnpm install --frozen-lockfile && pnpm run build

# Build Rust binary for current architecture
build-rust:
    cargo build --release

# Build everything locally (UI + Rust) and copy to out/
build: build-ui build-rust
    mkdir -p out
    cp target/release/ember-network-connect out/
    cp -r ui/build out/ui

# Build Docker image for current architecture
docker-build: build
    docker build -f Dockerfile.runtime \
        --build-context ui=out/ui \
        --build-context scripts=scripts \
        -t ember-network-connect:local \
        out

# Run the UI dev server with hot reload (uses mock API)
dev-ui:
    cd ui && pnpm dev

# Run the UI dev server connected to a real backend
# Usage: just dev-ui-backend http://192.168.1.100:80
dev-ui-backend backend_url:
    cd ui && VITE_BACKEND_URL={{backend_url}} pnpm dev

# Run the Rust binary (requires build first)
run: build
    ./target/release/ember-network-connect --ui-directory ui/build

# Clean all build artifacts
clean:
    rm -rf out target ui/build ui/node_modules

# Clean build artifacts but keep dependencies
clean-build:
    rm -rf out target ui/build

# Install UI dependencies
install-ui:
    cd ui && pnpm install

# Run lints
lint:
    cargo clippy --all-targets --all-features -- -D warnings
    cd ui && pnpm run lint

# Format code
fmt:
    cargo fmt
    cd ui && pnpm run lint -- --fix

# Check if everything builds without errors
check:
    cargo check
    cd ui && pnpm run build

# Build binary using Dockerfile.binary (tests CI build locally)
docker-build-binary arch="arm64":
    docker buildx build -f Dockerfile.binary \
        --platform linux/{{arch}} \
        --output type=local,dest=./out \
        .

# Create and push a git tag to trigger a release
# Usage: just release 4.12.0
release version:
    #!/usr/bin/env bash
    set -euo pipefail
    TAG="v{{version}}"
    echo "Creating tag: ${TAG}"
    git tag "${TAG}"
    git push origin "${TAG}"
    echo "Released ${TAG}"
