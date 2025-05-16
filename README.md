# Introduction

`nmcp` is a Rust-based Kubernetes operator that orchestrates Model Context Protocol (MCP) servers in Kubernetes environments. It manages server lifecycles through custom resources, automatically reconciles their state with underlying resources, and provides a **unified HTTP API gateway** for easy interaction with the MCP servers.

```yaml
# MCPServer.default.yaml
apiVersion: nmcp.nwrx.io/v1
kind: MCPServer
metadata:
  name: context7
  namespace: default
spec:
  image: mcp/context7:latest
  pool: default
  idleTimeout: 60s
  transport: 
    type: stdio
```

# Kubernetes Operator

The operator watches for `MCPServer` and `MCPPool` custom resources in the Kubernetes cluster. When an `MCPServer` resource is in the `Requested` phase, the operator creates a corresponding Pod and Service. The operator also manages the lifecycle of these resources, including termination of idle servers based on configured timeout periods.

```bash
# Start the operator.
nmcp operator
```

```bash
# Install the CRDs
nmcp export --type crd --resource pool | kubectl apply -f -
nmcp export --type crd --resource server | kubectl apply -f -
```

```bash
# Apply the MCPPool and MCPServer resources.
kubectl apply -f MCPPool.default.yaml
kubectl apply -f MCPServer.default.yaml
```

```bash
# List all MCP servers
kubectl get mcpservers
NAME         STATUS      AGE
context7     Idle        1m
```
---

# Gateway API

The gateway is a simple HTTP API server that exposes the MCP servers for management and monitoring. It listens for incoming requests and forwards them to the appropriate MCP server based on the transport method specified in the `MCPServer` resource.

```bash
# Start the gateway.
nmcp gateway
```

```bash
# List all MCP servers
curl -X GET http://localhost:8080/api/v1/servers
```

```bash
# Interact with a specific MCP server via SSE.
$ curl -X GET http://localhost:8080/api/v1/servers/context7/sse

# First chunk is the message endpoint.
event: endpoint
data: /api/v1/servers/context7/message
```

# Pools management

Servers are grouped into pools, which define limits on the number of servers that can be instantiated concurrently. The operator will not allow the creation of new servers if the pool's limit is reached. The operator also manages the lifecycle of these resources, including termination of idle servers based on configured timeout periods.

```yaml
# MCPPool.default.yaml
apiVersion: nmcp.nwrx.io/v1
kind: MCPPool
metadata:
  name: default
  namespace: default
spec:
  maxIdle: 2
  maxActive: 5
  defaultIdleTimeout: 60s
```

# Roadmap

The future development of `nmcp` is focused on the following key areas:

### Pool Management

- [ ] **Limits and Quotas**: Implement resource limits and quotas for MCP servers to ensure fair resource allocation and prevent abuse.
- [ ] **Default Values**: Introduce default values for MCP server configurations to simplify the setup process.
- [ ] **Non-image-based Servers**: Support for non-image-based MCP servers, allowing for `uvx`, `npx`, `hyper-mcp`, and other to be deployed.
- [ ] **Pool Autoscaling**: Automatic scaling of pools based on usage patterns and demand.
- [ ] **Resource Optimization**: Implement intelligent resource allocation based on historical usage patterns.

### Transport Compatibility
- [ ] **SSE Transport**: Implement full support for Server-Sent Events (SSE) transport between MCPServer and the gateway
- [ ] **Streamable HTTP (Pod to Gateway)**: Add support for [Streamable HTTP](https://modelcontextprotocol.io/specification/2025-03-26/basic/transports#streamable-http) communication from MCP server Pods to the gateway
- [ ] **Protocol Compliance**: Ensure strict adherence to the MCP specification for all transport methods.
- [ ] **Transport Fallback Strategy**: Implement automatic fallback mechanisms when preferred transport methods are unavailable.

### Transport Methods
- [x] **SSE Transport**: Expose SSE endpoints in the gateway API for client applications
- [ ] **Streamable HTTP**: Expose Streamable HTTP endpoints in the gateway API for client applications
- [ ] **WebSocket Transport**: Expose WebSocket endpoints in the gateway API for client applications
- [ ] **gRPC Transport**: Add support for gRPC-based communication for high-performance use cases.

### Runtime Expansion
- [ ] **Docker Runtime Support**: Enable the operator to manage MCP servers in Docker environments without requiring Kubernetes
- [ ] **Docker Transport**: Implement a specialized transport method optimized for Docker container communication
- [ ] **Serverless Deployment**: Support for deploying MCP servers in serverless environments (AWS Lambda, Azure Functions, etc.).
- [ ] **Edge Deployment**: Optimize for edge computing scenarios with limited resources.

### Observability & Monitoring
- [ ] **Prometheus Integration**: Add exporters for Prometheus metrics to monitor server usage, performance, and resource consumption of `MCPServer` and `MCPPool`
- [ ] **OpenTelemetry Support**: Implement distributed tracing with OpenTelemetry to track request flows across the system
- [ ] **Enhanced Logging**: Structured logging with configurable verbosity levels for improved troubleshooting.
- [ ] **Health Dashboards**: Pre-configured Grafana dashboards for monitoring system health.
- [ ] **Alerting Integration**: Set up alerting for critical system states and potential issues.

### Distribution
- [x] **Docker Image**: Create a Docker image for the NMCP operator and gateway, allowing for easy deployment in various environments
- [ ] **Helm Chart**: Create a Helm chart for easy deployment and management of the NMCP operator and gateway in Kubernetes environments
- [ ] **Kustomize Support**: Provide Kustomize overlays for different environments (e.g., dev, staging, production) to simplify deployment configurations
- [ ] **Terraform Provider**: Develop a Terraform provider for managing MCP servers and pools, enabling infrastructure as code (IaC) capabilities
- [ ] **Cloud Marketplace Offerings**: Package for deployment through cloud marketplaces (AWS, Azure, GCP).

### Security Improvements
- [ ] **Network Policies**: Define and enforce Kubernetes NetworkPolicies to secure communication between components
- [ ] **Image Security**: Add configurable allowlists/denylists for MCP server images to enhance deployment security
- [ ] **Authentication & Authorization**: Implement robust auth mechanisms for the API gateway.
- [ ] **Role-Based Access Control**: Fine-grained permissions for different user roles.
- [ ] **Secret Management**: Integration with external secret stores (Vault, cloud provider solutions).
- [ ] **TLS Everywhere**: Enforce encrypted communication between all components.

### Resilience & High Availability
- [ ] **Controller Redundancy**: Support for running multiple controller instances for high availability.
- [ ] **Graceful Degradation**: Maintain core functionality when dependent services are unavailable.
- [ ] **Automatic Recovery**: Self-healing capabilities for common failure scenarios.
- [ ] **Connection Pooling**: Implement efficient connection management for better resource utilization.
- [ ] **Circuit Breaking**: Prevent cascading failures with circuit breaker patterns.

### Performance Optimization
- [ ] **Efficient Status Updates**: Reduce API server pressure by optimizing status update frequency.
- [ ] **Resource Efficiency**: Improve memory and CPU utilization of the operator itself.
- [ ] **Batch Processing**: Handle operations in batches for better throughput.
- [ ] **Caching Strategy**: Implement intelligent caching to reduce latency and resource usage.
- [ ] **Horizontal Scaling**: Support for distributing gateway load across multiple instances.
