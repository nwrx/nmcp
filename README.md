# unmcp

A Kubernetes operator for managing Model Context Protocol (MCP) servers in Kubernetes environments.

## Overview

`unmcp` is a Rust-based Kubernetes operator that manages the lifecycle of MCP servers in Kubernetes. It watches for MCPServer and MCPPool custom resources and reconciles their state with the underlying Kubernetes resources (Pods and Services).

The operator:
- Monitors MCPServer and MCPPool resources
- Creates and manages Pods and Services for MCP servers
- Scales servers based on pool specifications
- Handles server lifecycle based on configuration (idle timeout, etc.)
- Exposes an HTTP API for management and monitoring

## Usage

```bash
# Run the operator in a specific namespace
unmcp operator --namespace default

# Run only the API server with a custom port
unmcp server --host 0.0.0.0 --port 3000

# Show help
unmcp --help
```

### Command-line options

| Option | Description | Default |
|--------|-------------|---------|
| `--namespace` | Kubernetes namespace to watch for resources | `default` |
| `--port` | HTTP server port for the operator API | `8080` |
| `--host` | Host address to bind the API server to | `127.0.0.1` |
| `--disable-operator` | Disable the operator component | `false` |
| `--disable-api` | Disable the API server component | `false` |
| `--log-level` | Log level (debug, info, warn, error) | `info` |
| `--kubeconfig` | Path to kubeconfig file | Uses in-cluster config if not specified |

## Features

- **Custom Resource Management**: Watches and reconciles MCPServer and MCPPool custom resources
- **Automated Lifecycle**: Manages the complete lifecycle of MCP servers, from creation to termination
- **Resource Efficiency**: Terminates idle servers based on configured timeout periods
- **Pool-based Management**: Groups servers into pools for easier management and lifecycle handling
- **Resource Limits and Requests**: Configurable CPU and memory limits for each server
- **API Exposure**: HTTP API for management, monitoring, and integration
- **Transport Options**: Supports multiple transport methods (SSE, STDIO)

The operator will automatically:
1. Create a Pod and Service for the MCP server
3. Monitor the server's activity
4. Terminate the server if it remains idle for the configured timeout period

## API Endpoints

The operator exposes several HTTP API endpoints for managing and monitoring MCP resources:

```
# Pool Management
GET /api/v1/pools                    # List all available pools in the namespace
GET /api/v1/pools/{name}             # Get details for a specific pool
POST /api/v1/pools                   # Create a new pool
PUT /api/v1/pools/{name}             # Update the configuration of an existing pool
DELETE /api/v1/pools/{name}          # Delete a pool and all its servers

# Server Management
GET /api/v1/servers                  # List all servers in the namespace
GET /api/v1/servers/{uid}            # Get details for a specific server
POST /api/v1/servers                 # Create a new server
PUT /api/v1/servers/{uid}            # Update an existing server
DELETE /api/v1/servers/{uid}         # Delete a server

# Interaction with Servers
GET /api/v1/servers/{uid}/sse        # Get the SSE stream for a specific server
POST /api/v1/servers/{uid}/message  # Send input to a server's stdin

# Metrics
GET /api/v1/metrics                  # Get operator metrics (active servers, etc.)
GET /api/v1/metrics/pools/{name}     # Get metrics for a specific pool
GET /api/v1/metrics/servers/{name}   # Get metrics for a specific server

# Monitoring (Prometheus/Grafana Integration)
GET /metrics                         # Prometheus-compatible metrics endpoint with all metrics

# Health and Status
GET /health                          # Operator health check
GET /ready                           # Operator readiness check
```

Example response for listing servers:

```json
{
  "items": [
    {
      "name": "time-service",
      "namespace": "default",
      "pool": "default",
      "status": {
        "isRunning": true,
        "isIdle": false,
        "totalRequests": 42,
        "currentConnections": 3,
        "lastRequestAt": "2025-05-02T10:15:30Z"
      }
    },
    {
      "name": "weather-service",
      "namespace": "default",
      "pool": "default",
      "status": {
        "isRunning": true,
        "isIdle": true,
        "totalRequests": 7,
        "currentConnections": 0,
        "lastRequestAt": "2025-05-02T09:45:12Z"
      }
    }
  ]
}
```

## Architecture

The operator follows the Kubernetes operator pattern:

1. Watches for changes to MCPServer and MCPPool custom resources
2. Reconciles the current state with the desired state
3. Creates, updates, or deletes the underlying Kubernetes resources (Pods, Services)
4. Updates the status of the custom resources

The HTTP server exposes endpoints for management and monitoring of the MCP resources.