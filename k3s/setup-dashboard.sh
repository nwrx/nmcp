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

# Get token for admin-user
echo "=== Getting dashboard access token ==="
if kubectl -n kubernetes-dashboard get secret admin-user &>/dev/null; then
    # Delete existing secret if it exists
    kubectl -n kubernetes-dashboard delete secret admin-user
fi

kubectl -n kubernetes-dashboard create token admin-user > dashboard-token.txt
TOKEN=$(cat dashboard-token.txt)

echo ""
echo "=== Dashboard Access Information ==="
echo "Dashboard URL: http://localhost:8001/api/v1/namespaces/kubernetes-dashboard/services/https:kubernetes-dashboard:/proxy/"
echo ""
echo "Access Token (also saved to dashboard-token.txt):"
echo "$TOKEN"
echo ""

echo "=== Starting kubectl proxy ==="
echo "Starting kubectl proxy in the background on port 8001"
echo "Press Ctrl+C to stop the proxy when you're done"
kubectl proxy --address='0.0.0.0' --port=8001 --accept-hosts='.*'