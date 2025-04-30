# Default recipe to display help
default:
    @just --list

# Start the k3s cluster using docker-compose
k3s-start:
    cd k3s && docker-compose up -d

# Stop the k3s cluster
k3s-stop:
    cd k3s && docker-compose down -v

# Restart the k3s cluster
k3s-restart: k3s-stop k3s-start

# Setup and start the k8s dashboard
k3s-dashboard: k3s-start
    #!/usr/bin/env bash
    cd k3s && ./setup-dashboard.sh

##########################################

# Run all Rust tests
test:
    cargo test --workspace

# Run tests for specific project
test-crds:
    cargo test -p unmcp-crds

test-operator:
    cargo test -p unmcp-operator

test-utils:
    cargo test -p unmcp-test-utils

# Clean all build artifacts
clean:
    cargo clean
    rm -rf k3s/kubeconfig/*
    @echo "Cleaned build artifacts and kubeconfig"

##########################################

# Export CRD schemas
crd-schemas:
    #!/usr/bin/env bash
    cargo run -- schema pool --format json > k3s/schema-crd-pool.json
    cargo run -- schema server --format json > k3s/schema-crd-server.json

# Install CRDs (cargo run -- crd pool --format yaml)
crd-install:
    #!/usr/bin/env bash
    cargo run -- crd pool --format yaml | kubectl apply -f -
    cargo run -- crd server --format yaml | kubectl apply -f -
