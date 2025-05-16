export KUBECONFIG := `pwd`+"/kube/kubeconfig/kubeconfig.yaml"
export DOCKER_BUILDKIT := "1"

# Default recipe to display help
default:
    @just --list

##########################################

build:
    cargo build

##########################################

# Export CRD schemas
kube-crd-export: build
    ./target/debug/nmcp export --type crd --resource pool > ./k3s/pool.json
    ./target/debug/nmcp export --type crd --resource server > ./k3s/server.json

kube-crd-uninstall: build
    ./target/debug/nmcp export --type crd --resource pool | kubectl delete -f - || true
    ./target/debug/nmcp export --type crd --resource server --format yaml | kubectl delete -f - || true

kube-crd-install: kube-crd-uninstall
    ./target/debug/nmcp export --type crd --resource pool --format yaml | kubectl apply -f -
    ./target/debug/nmcp export --type crd --resource server --format yaml | kubectl apply -f -

##########################################

# Start the Kubernetes cluster.
kube-start:
    cd kube && docker-compose up --detach

# Setup and start the k8s dashboard
kube-setup: kube-start
    #!/usr/bin/env bash
    set -e
    until [ -f "$KUBECONFIG" ] && kubectl --kubeconfig="$KUBECONFIG" get nodes &>/dev/null; do
        echo "Waiting for kubeconfig ($KUBECONFIG) to be created..."
        sleep 5
    done
    kubectl --kubeconfig="$KUBECONFIG" wait \
        --for=condition=ready node \
        --all \
        --timeout=60s

kube:
    @just kube-start
    @just kube-setup
    @just kube-crd-install

##########################################

# Start the operator
operator:
    cargo watch -s 'clear && cargo run -- operator'

# Start the server
gateway:
    cargo watch -s 'clear && cargo run -- gateway --port 3000'

##########################################

# Docker image building and pushing
# Build the static Docker image with musl
docker-build registry='' tag='latest':
    docker build -t {{registry}}nmcp:{{tag}} .

# Push the Docker images to a registry
docker-push registry='' tag='latest':
    docker push {{registry}}nmcp:{{tag}}

# Build and push in one command
docker-build-push registry='' tag='latest':
    just docker-build {{registry}} {{tag}}
    just docker-push {{registry}} {{tag}}

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