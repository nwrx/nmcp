# unmcp-server

Dynamic MCP execution server

## Example

```bash

# Using Node Package Manager
unmcp-server \
  --runtime npx \
  --package @modelcontextprotocol/server-github \
  --env GITHUB_TOKEN=your_github_token \
  --port 8080

# Using Python Unicorn
unmcp-server \
  --runtime uvx \
  --package modelcontextprotocol/server-github \
  --env GITHUB_TOKEN=your_github_token \
  --port 8080

# Using HyperMCP
unmcp-server \
  --runtime hyper-mcp \
  --path oci://ghcr.io/tuananh/github-plugin:latest \
  --env GITHUB_TOKEN=your_github_token \
  --port 8080
```

## Features

- Support multiple runtimes: Node.js, Python, HyperMCP and their underlying package/dependency managers.
- Automatically download and install the package if it is not available locally.
- Cache the package to speed up subsequent runs.
