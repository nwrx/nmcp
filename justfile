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
k3s: k3s-start
    #!/usr/bin/env bash
    cd k3s && ./setup-dashboard.sh

##########################################

# Run all Rust tests
test:
    cargo test --workspace

# Clean all build artifacts
clean:
    cargo clean
    rm -rf k3s/kubeconfig/*
    @echo "Cleaned build artifacts and kubeconfig"

# Build all projects
build:
    #!/usr/bin/env bash
    cargo build --workspace

##########################################

# Export CRD schemas
crd-export: build
    #!/usr/bin/env bash
    ./target/debug/unmcp export --type crd --resource pool --format json > ./k3s/pool.json
    ./target/debug/unmcp export --type crd --resource server --format json > ./k3s/server.json

crd-uninstall: build
    #!/usr/bin/env bash
    ./target/debug/unmcp export --type crd --resource pool --format yaml | kubectl delete -f - || true
    ./target/debug/unmcp export --type crd --resource server --format yaml | kubectl delete -f - || true

# Install CRDs (./target/debug/unmcp-crds crd pool --format yaml)
crd-install: crd-uninstall
    #!/usr/bin/env bash
    ./target/debug/unmcp export --type crd --resource pool --format yaml | kubectl apply -f -
    ./target/debug/unmcp export --type crd --resource server --format yaml | kubectl apply -f -

##########################################

# Start the operator
operator:
    cargo watch -x 'run -- operator'

# Start the server
server:
    cargo watch -x 'run -- server --port 3000'

##########################################

example:
    tsx ./scripts/testMcpServer.ts