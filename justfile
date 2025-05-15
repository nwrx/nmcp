export KUBECONFIG := `pwd`+"/kube/kubeconfig/kubeconfig.yaml"

# Default recipe to display help
default:
    @just --list

##########################################

build:
    cargo build

##########################################

# Export CRD schemas
kube-crd-export: build
    ./target/debug/ncmp export --type crd --resource pool > ./k3s/pool.json
    ./target/debug/ncmp export --type crd --resource server > ./k3s/server.json

kube-crd-uninstall: build
    ./target/debug/ncmp export --type crd --resource pool | kubectl delete -f - || true
    ./target/debug/ncmp export --type crd --resource server --format yaml | kubectl delete -f - || true

kube-crd-install: kube-crd-uninstall
    ./target/debug/ncmp export --type crd --resource pool --format yaml | kubectl apply -f -
    ./target/debug/ncmp export --type crd --resource server --format yaml | kubectl apply -f -

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
