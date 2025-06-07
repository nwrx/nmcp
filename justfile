export KUBECONFIG := `pwd`+"/examples/kubernetes/kubeconfig/kubeconfig.yaml"
export DOCKER_BUILDKIT := "1"
export RUST_BACKTRACE := "1"

# Default recipe to display help
default:
    @just --list

##########################################

build:
    nix build .#default

build-docker:
    nix build .#dockerImage

# Update the cargo version, create a new git tag, and push the changes
release type='minor':
    cargo release \
        --no-publish \
        --sign-commit \
        --sign-tag \
        --no-push \
        {{type}}

##########################################

# Start the Kubernetes cluster.
kube-start:
    cd examples/kubernetes && docker-compose up --detach

# Setup and start the k8s dashboard
kube-setup: kube-start
    #!/usr/bin/env bash
    set -e
    until [ -f "$KUBECONFIG" ] && kubectl --kubeconfig="$KUBECONFIG" get nodes &>/dev/null; do
        echo "Waiting for kubeconfig ($KUBECONFIG) to be created..."
        sleep 5
    done

    # Wait for the cluster to be ready.
    kubectl --kubeconfig="$KUBECONFIG" wait \
        --for=condition=ready node \
        --all \
        --timeout=60s

kube-create-crd:
    cargo run -- export -r server -t crd -f yaml > examples/kubernetes/manifests/CRD.MCPServer.yaml
    cargo run -- export -r pool -t crd -f yaml > examples/kubernetes/manifests/CRD.MCPPool.yaml

# Apply the manifests
kube-apply:
    kubectl --kubeconfig="$KUBECONFIG" apply \
        --filename examples/kubernetes/manifests

kube:
    @just kube-start
    @just kube-setup
    @just kube-create-crd
    @just kube-apply

##########################################

# Start the operator
operator:
    cargo watch -s 'clear && cargo run -- operator --log-level trace --log-format detailed --show-backtrace'

# Start the gateway API
gateway:
    cargo watch -s 'clear && cargo run -- gateway --port 8080 --log-level trace --log-format detailed --show-backtrace'

# Start the manager API
manager:
    cargo watch -s 'clear && cargo run -- manager --port 8081 --log-level trace --log-format detailed --show-backtrace'

##########################################

# Build the Docker image
docker-build:
    nix build .#dockerImage
    docker load < result
    rm result

# Run the NMCP operator in Docker
docker-run-operator kubeconfig_path=KUBECONFIG:
    @just docker-build
    docker run --rm \
        --volume {{kubeconfig_path}}:/app/kubeconfig.yaml \
        nwrx/nmcp:next operator \
        --kubeconfig /app/kubeconfig.yaml

# Run the NMCP gateway in Docker
docker-run-gateway kubeconfig_path=KUBECONFIG port='8080':
    @just docker-build
    docker run --rm \
        --volume {{kubeconfig_path}}:/app/kubeconfig.yaml \
        -p {{port}}:{{port}} \
        nwrx/nmcp:next gateway \
        --kubeconfig /app/kubeconfig.yaml \
        --port {{port}} \
        --host 0.0.0.0
