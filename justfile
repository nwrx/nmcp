export KUBECONFIG := `pwd`+"/examples/kubernetes/kubeconfig/kubeconfig.yaml"
export DOCKER_BUILDKIT := "1"

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
    cd examples && docker-compose up --detach

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

    # Apply the manifests
    kubectl --kubeconfig="$KUBECONFIG" apply \
        --filename examples/kubernetes/manifests

kube:
    @just kube-start
    @just kube-setup

##########################################

# Start the operator
operator:
    cargo watch -s 'clear && cargo run -- operator'

# Start the server
gateway:
    cargo watch -s 'clear && cargo run -- gateway --port 8080'

##########################################

# Run the NMCP operator in Docker
docker-run-operator registry='' tag='latest' kubeconfig_path=KUBECONFIG:
    docker run --rm \
        --volume {{kubeconfig_path}}:/app/kubeconfig.yaml \
        {{registry}}nmcp:{{tag}} operator \
        --kubeconfig /app/kubeconfig.yaml

# Run the NMCP gateway in Docker
docker-run-gateway registry='' tag='latest' kubeconfig_path=KUBECONFIG port='8080':
    docker run --rm \
        --volume {{kubeconfig_path}}:/app/kubeconfig.yaml \
        -p {{port}}:{{port}} \
        {{registry}}nmcp:{{tag}} gateway \
        --kubeconfig /app/kubeconfig.yaml \
        --port {{port}} \
        --host 0.0.0.0
