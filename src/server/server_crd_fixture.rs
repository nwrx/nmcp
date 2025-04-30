#[allow(dead_code)]
pub const MCP_SERVER_CRD_FIXTURE: &str = r##"
{
  "apiVersion": "apiextensions.k8s.io/v1",
  "kind": "CustomResourceDefinition",
  "metadata": {
    "name": "mcpservers.unmcp.dev"
  },
  "spec": {
    "group": "unmcp.dev",
    "names": {
      "categories": [],
      "kind": "MCPServer",
      "plural": "mcpservers",
      "shortNames": [
        "mcp"
      ],
      "singular": "mcpserver"
    },
    "scope": "Namespaced",
    "versions": [
      {
        "additionalPrinterColumns": [
          {
            "jsonPath": ".spec.pool",
            "name": "Pool",
            "type": "string"
          },
          {
            "jsonPath": ".status.phase",
            "name": "Status",
            "type": "string"
          },
          {
            "jsonPath": ".metadata.creationTimestamp",
            "name": "Age",
            "type": "date"
          },
          {
            "jsonPath": ".spec.metadata.serverType",
            "name": "Type",
            "type": "string"
          }
        ],
        "name": "v1",
        "schema": {
          "openAPIV3Schema": {
            "description": "Auto-generated derived type for MCPServerSpec via `CustomResource`",
            "properties": {
              "spec": {
                "description": "MCPServer custom resource definition",
                "properties": {
                  "image": {
                    "default": "mcp/time:latest",
                    "description": "Container image to use",
                    "type": "string"
                  },
                  "livenessProbe": {
                    "description": "Liveness probe configuration",
                    "nullable": true,
                    "properties": {
                      "httpGet": {
                        "description": "HTTP get probe",
                        "properties": {
                          "path": {
                            "description": "Path to probe",
                            "type": "string"
                          },
                          "port": {
                            "description": "Port to probe",
                            "format": "int32",
                            "type": "integer"
                          }
                        },
                        "required": [
                          "path",
                          "port"
                        ],
                        "type": "object"
                      },
                      "initialDelaySeconds": {
                        "default": 5,
                        "description": "Initial delay seconds",
                        "format": "int32",
                        "type": "integer"
                      },
                      "periodSeconds": {
                        "default": 10,
                        "description": "Period seconds",
                        "format": "int32",
                        "type": "integer"
                      }
                    },
                    "required": [
                      "httpGet"
                    ],
                    "type": "object"
                  },
                  "metadata": {
                    "default": {
                      "capabilities": [],
                      "serverType": null
                    },
                    "description": "Server metadata",
                    "properties": {
                      "capabilities": {
                        "default": [],
                        "description": "Server capabilities",
                        "items": {
                          "type": "string"
                        },
                        "type": "array"
                      },
                      "serverType": {
                        "description": "Server type",
                        "nullable": true,
                        "type": "string"
                      }
                    },
                    "type": "object"
                  },
                  "networking": {
                    "default": {
                      "cors": null,
                      "exposeExternally": false,
                      "port": 0,
                      "protocol": ""
                    },
                    "description": "Network configuration",
                    "properties": {
                      "cors": {
                        "description": "CORS configuration",
                        "nullable": true,
                        "properties": {
                          "allowOrigins": {
                            "description": "Allowed origins",
                            "items": {
                              "type": "string"
                            },
                            "type": "array"
                          }
                        },
                        "required": [
                          "allowOrigins"
                        ],
                        "type": "object"
                      },
                      "exposeExternally": {
                        "default": false,
                        "description": "Whether to expose externally, will either create a LoadBalancer or ClusterIP service depending on the value of this field. Keep in mind that exposing externally may incur costs.",
                        "type": "boolean"
                      },
                      "port": {
                        "default": 8080,
                        "description": "Server port",
                        "format": "int32",
                        "type": "integer"
                      },
                      "protocol": {
                        "default": "HTTP",
                        "description": "Protocol (HTTP or HTTPS)",
                        "type": "string"
                      }
                    },
                    "type": "object"
                  },
                  "pool": {
                    "default": "",
                    "description": "Reference to McpPool resource",
                    "type": "string"
                  },
                  "readinessProbe": {
                    "description": "Readiness probe configuration",
                    "nullable": true,
                    "properties": {
                      "httpGet": {
                        "description": "HTTP get probe",
                        "properties": {
                          "path": {
                            "description": "Path to probe",
                            "type": "string"
                          },
                          "port": {
                            "description": "Port to probe",
                            "format": "int32",
                            "type": "integer"
                          }
                        },
                        "required": [
                          "path",
                          "port"
                        ],
                        "type": "object"
                      },
                      "initialDelaySeconds": {
                        "default": 5,
                        "description": "Initial delay seconds",
                        "format": "int32",
                        "type": "integer"
                      },
                      "periodSeconds": {
                        "default": 10,
                        "description": "Period seconds",
                        "format": "int32",
                        "type": "integer"
                      }
                    },
                    "required": [
                      "httpGet"
                    ],
                    "type": "object"
                  },
                  "resources": {
                    "default": {
                      "limits": null,
                      "requests": null
                    },
                    "description": "Resource requirements",
                    "properties": {
                      "limits": {
                        "description": "Resource limits",
                        "nullable": true,
                        "properties": {
                          "cpu": {
                            "description": "CPU quantity",
                            "nullable": true,
                            "type": "string"
                          },
                          "memory": {
                            "description": "Memory quantity",
                            "nullable": true,
                            "type": "string"
                          }
                        },
                        "type": "object"
                      },
                      "requests": {
                        "description": "Resource requests",
                        "nullable": true,
                        "properties": {
                          "cpu": {
                            "description": "CPU quantity",
                            "nullable": true,
                            "type": "string"
                          },
                          "memory": {
                            "description": "Memory quantity",
                            "nullable": true,
                            "type": "string"
                          }
                        },
                        "type": "object"
                      }
                    },
                    "type": "object"
                  },
                  "securityContext": {
                    "description": "Security context",
                    "nullable": true,
                    "properties": {
                      "allowPrivilegeEscalation": {
                        "default": false,
                        "description": "Allow privilege escalation",
                        "type": "boolean"
                      },
                      "capabilities": {
                        "description": "Capabilities",
                        "nullable": true,
                        "properties": {
                          "drop": {
                            "description": "Capabilities to drop",
                            "items": {
                              "type": "string"
                            },
                            "type": "array"
                          }
                        },
                        "required": [
                          "drop"
                        ],
                        "type": "object"
                      },
                      "runAsNonRoot": {
                        "default": true,
                        "description": "Run as non-root",
                        "type": "boolean"
                      },
                      "runAsUser": {
                        "description": "Run as user ID",
                        "format": "int64",
                        "nullable": true,
                        "type": "integer"
                      },
                      "seccompProfile": {
                        "description": "Seccomp profile",
                        "nullable": true,
                        "properties": {
                          "type": {
                            "description": "Profile type",
                            "type": "string"
                          }
                        },
                        "required": [
                          "type"
                        ],
                        "type": "object"
                      }
                    },
                    "type": "object"
                  },
                  "server": {
                    "description": "Server command configuration",
                    "properties": {
                      "args": {
                        "default": [],
                        "description": "Command arguments",
                        "items": {
                          "type": "string"
                        },
                        "type": "array"
                      },
                      "command": {
                        "description": "Main command to execute",
                        "type": "string"
                      },
                      "env": {
                        "additionalProperties": {
                          "type": "string"
                        },
                        "default": {},
                        "description": "Environment variables",
                        "type": "object"
                      }
                    },
                    "required": [
                      "command"
                    ],
                    "type": "object"
                  },
                  "storage": {
                    "default": {
                      "ephemeral": false,
                      "volumeSize": null
                    },
                    "description": "Storage configuration",
                    "properties": {
                      "ephemeral": {
                        "default": true,
                        "description": "Whether storage is ephemeral",
                        "type": "boolean"
                      },
                      "volumeSize": {
                        "description": "Volume size",
                        "nullable": true,
                        "type": "string"
                      }
                    },
                    "type": "object"
                  },
                  "version": {
                    "default": "1.0.0",
                    "description": "Server version",
                    "type": "string"
                  }
                },
                "required": [
                  "server"
                ],
                "type": "object"
              },
              "status": {
                "description": "MCPServer status",
                "nullable": true,
                "properties": {
                  "conditions": {
                    "default": [],
                    "description": "Status conditions",
                    "items": {
                      "properties": {
                        "lastTransitionTime": {
                          "format": "date-time",
                          "nullable": true,
                          "type": "string"
                        },
                        "message": {
                          "nullable": true,
                          "type": "string"
                        },
                        "reason": {
                          "nullable": true,
                          "type": "string"
                        },
                        "status": {
                          "type": "string"
                        },
                        "type": {
                          "type": "string"
                        }
                      },
                      "required": [
                        "status",
                        "type"
                      ],
                      "type": "object"
                    },
                    "type": "array"
                  },
                  "metrics": {
                    "description": "Server metrics",
                    "nullable": true,
                    "properties": {
                      "activeConnections": {
                        "description": "Active connections",
                        "format": "int32",
                        "nullable": true,
                        "type": "integer"
                      },
                      "cpuUsage": {
                        "description": "CPU usage",
                        "nullable": true,
                        "type": "string"
                      },
                      "memoryUsage": {
                        "description": "Memory usage",
                        "nullable": true,
                        "type": "string"
                      },
                      "requestCount": {
                        "description": "Request count",
                        "format": "int64",
                        "nullable": true,
                        "type": "integer"
                      }
                    },
                    "type": "object"
                  },
                  "phase": {
                    "description": "Server phase",
                    "nullable": true,
                    "type": "string"
                  },
                  "serverEndpoint": {
                    "description": "Server endpoint URL",
                    "nullable": true,
                    "type": "string"
                  },
                  "serverUuid": {
                    "description": "Server UUID",
                    "nullable": true,
                    "type": "string"
                  },
                  "startTime": {
                    "description": "Start time",
                    "format": "date-time",
                    "nullable": true,
                    "type": "string"
                  }
                },
                "type": "object"
              }
            },
            "required": [
              "spec"
            ],
            "title": "MCPServer",
            "type": "object"
          }
        },
        "served": true,
        "storage": true,
        "subresources": {
          "status": {}
        }
      }
    ]
  }
}
"##;