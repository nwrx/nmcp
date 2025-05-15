#!/usr/bin/env bash
set -e

SCRIPT_DIR=$(cd $(dirname $0); pwd)
KUBECONFIG="$SCRIPT_DIR/kubeconfig/kubeconfig.yaml"
K3S_SERVER_CONTAINER="unmcp-k3s-server"

# Ensure the kubeconfig directory exists
mkdir -p "$SCRIPT_DIR/kubeconfig"

echo "=== Waiting for K3s to start ==="
until [ -f "$KUBECONFIG" ] && kubectl --kubeconfig="$KUBECONFIG" get nodes &>/dev/null; do
    echo "Waiting for kubeconfig to be accessible..."
    sleep 5
done

# Set KUBECONFIG environment variable
export KUBECONFIG="$KUBECONFIG"
echo "=== K3s is ready ==="

# Wait for all nodes to be ready
echo "=== Waiting for all nodes to be ready ==="
kubectl --kubeconfig="$KUBECONFIG" wait --for=condition=ready node --all --timeout=120s

# Display cluster information
echo "=== Cluster Information ==="
kubectl --kubeconfig="$KUBECONFIG" get nodes
echo ""
