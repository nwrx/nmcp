# Default recipe to display help
default:
    @just --list

# Start the Kubernetes cluster using docker-compose
k3s-start:
    cd k3s && docker-compose up -d

# Stop the Kubernetes cluster
k3s-stop:
    cd k3s && docker-compose down -v

# Restart the Kubernetes cluster
k3s-restart: k3s-stop k3s-start

# Setup and start the k8s dashboard
k3s: k3s-start
    ./kube/setup.sh

##########################################

build:
    cargo build

# Export CRD schemas
crd-export: build
    ./target/debug/unmcp export --type crd --resource pool --format json > ./k3s/pool.json
    ./target/debug/unmcp export --type crd --resource server --format json > ./k3s/server.json

crd-uninstall: build
    ./target/debug/unmcp export --type crd --resource pool --format yaml | kubectl delete -f - || true
    ./target/debug/unmcp export --type crd --resource server --format yaml | kubectl delete -f - || true

# Install CRDs (./target/debug/unmcp-crds crd pool --format yaml)
crd-install: crd-uninstall
    ./target/debug/unmcp export --type crd --resource pool --format yaml | kubectl apply -f -
    ./target/debug/unmcp export --type crd --resource server --format yaml | kubectl apply -f -

##########################################

# Start the operator
operator:
    cargo watch -s 'clear && cargo run -- operator'

# Start the server
gateway:
    cargo watch -s 'clear && cargo run -- gateway --port 3000'
