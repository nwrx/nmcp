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

# Deploy the Kubernetes Dashboard
echo "=== Deploying Kubernetes Dashboard ==="
kubectl apply -f https://raw.githubusercontent.com/kubernetes/dashboard/v2.7.0/aio/deploy/recommended.yaml

# Create dashboard admin user and role binding
echo "=== Creating dashboard admin user ==="
cat <<EOF | kubectl apply -f -
apiVersion: v1
kind: ServiceAccount
metadata:
  name: admin-user
  namespace: kubernetes-dashboard
---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRoleBinding
metadata:
  name: admin-user
roleRef:
  apiGroup: rbac.authorization.k8s.io
  kind: ClusterRole
  name: cluster-admin
subjects:
- kind: ServiceAccount
  name: admin-user
  namespace: kubernetes-dashboard
EOF

# Wait for dashboard to be ready
echo "=== Waiting for dashboard to be ready ==="
# Add a short delay to give time for pods to be created
sleep 10

# Check if dashboard pods exist and get the proper selector
echo "Checking for dashboard pods..."
if kubectl get pods -n kubernetes-dashboard -o name | grep -q "pod/kubernetes-dashboard"; then
  # Wait for dashboard pod to be ready
  echo "Found dashboard pod, waiting for it to be ready..."
  kubectl wait \
    --namespace kubernetes-dashboard \
    --for=condition=ready pod \
    --selector=k8s-app=kubernetes-dashboard \
    --timeout=180s || echo "Warning: Timed out waiting for dashboard pod, but will continue"
else
  echo "Dashboard pod not found with expected selector. Listing available pods:"
  kubectl get pods -n kubernetes-dashboard
  echo "Continuing with setup anyway..."
fi

# Also wait for the metrics-scraper if it exists
if kubectl get pods -n kubernetes-dashboard -o name | grep -q "pod/dashboard-metrics-scraper"; then
  echo "Waiting for metrics-scraper pod to be ready..."
  kubectl wait \
    --namespace kubernetes-dashboard \
    --for=condition=ready pod \
    --selector=k8s-app=dashboard-metrics-scraper \
    --timeout=60s || echo "Warning: Timed out waiting for metrics-scraper, but will continue"
fi

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