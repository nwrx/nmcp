#[allow(dead_code)]
pub const MCP_POOL_CRD_FIXTURE: &str = r#"
{
  "apiVersion": "apiextensions.k8s.io/v1",
  "kind": "CustomResourceDefinition",
  "metadata": {
    "name": "mcppools.unmcp.dev"
  },
  "spec": {
    "group": "unmcp.dev",
    "names": {
      "categories": [],
      "kind": "MCPPool",
      "plural": "mcppools",
      "shortNames": [
        "mcpp"
      ],
      "singular": "mcppool"
    },
    "scope": "Namespaced",
    "versions": [
      {
        "additionalPrinterColumns": [
          {
            "jsonPath": ".status.availableServers",
            "name": "Available",
            "type": "integer"
          },
          {
            "jsonPath": ".status.inUseServers",
            "name": "In Use",
            "type": "integer"
          },
          {
            "jsonPath": ".metadata.creationTimestamp",
            "name": "Age",
            "type": "date"
          }
        ],
        "name": "v1",
        "schema": {
          "openAPIV3Schema": {
            "description": "Auto-generated derived type for McpPoolSpec via `CustomResource`",
            "properties": {
              "spec": {
                "description": "McpPool custom resource definition",
                "properties": {
                  "affinity": {
                    "description": "Affinity configuration",
                    "nullable": true,
                    "properties": {
                      "podAntiAffinity": {
                        "description": "Pod anti-affinity",
                        "nullable": true,
                        "properties": {
                          "preferredDuringSchedulingIgnoredDuringExecution": {
                            "description": "Preferred during scheduling, ignored during execution",
                            "items": {
                              "description": "Weighted pod affinity term",
                              "properties": {
                                "podAffinityTerm": {
                                  "description": "Pod affinity term",
                                  "properties": {
                                    "labelSelector": {
                                      "description": "Label selector",
                                      "properties": {
                                        "matchLabels": {
                                          "additionalProperties": {
                                            "type": "string"
                                          },
                                          "description": "Match labels",
                                          "type": "object"
                                        }
                                      },
                                      "required": [
                                        "matchLabels"
                                      ],
                                      "type": "object"
                                    },
                                    "topologyKey": {
                                      "description": "Topology key",
                                      "type": "string"
                                    }
                                  },
                                  "required": [
                                    "labelSelector",
                                    "topologyKey"
                                  ],
                                  "type": "object"
                                },
                                "weight": {
                                  "description": "Weight",
                                  "format": "int32",
                                  "type": "integer"
                                }
                              },
                              "required": [
                                "podAffinityTerm",
                                "weight"
                              ],
                              "type": "object"
                            },
                            "nullable": true,
                            "type": "array"
                          }
                        },
                        "type": "object"
                      }
                    },
                    "type": "object"
                  },
                  "autoscaling": {
                    "default": {
                      "enabled": false,
                      "scaleDownCooldown": 0,
                      "scaleUpCooldown": 0,
                      "targetCpuUtilization": 0,
                      "targetMemoryUtilization": 0
                    },
                    "description": "Auto-scaling configuration",
                    "properties": {
                      "enabled": {
                        "default": false,
                        "description": "Whether auto-scaling is enabled",
                        "type": "boolean"
                      },
                      "scaleDownCooldown": {
                        "default": 300,
                        "description": "Scale down cooldown in seconds",
                        "format": "int32",
                        "type": "integer"
                      },
                      "scaleUpCooldown": {
                        "default": 60,
                        "description": "Scale up cooldown in seconds",
                        "format": "int32",
                        "type": "integer"
                      },
                      "targetCpuUtilization": {
                        "default": 70,
                        "description": "Target CPU utilization percentage",
                        "format": "int32",
                        "type": "integer"
                      },
                      "targetMemoryUtilization": {
                        "default": 80,
                        "description": "Target memory utilization percentage",
                        "format": "int32",
                        "type": "integer"
                      }
                    },
                    "type": "object"
                  },
                  "cleanupPolicy": {
                    "default": {
                      "deleteOrphanedServers": false,
                      "terminateGracePeriod": 0
                    },
                    "description": "Cleanup policy",
                    "properties": {
                      "deleteOrphanedServers": {
                        "default": true,
                        "description": "Whether to delete orphaned servers",
                        "type": "boolean"
                      },
                      "terminateGracePeriod": {
                        "default": 30,
                        "description": "Grace period for termination",
                        "format": "int32",
                        "type": "integer"
                      }
                    },
                    "type": "object"
                  },
                  "maxServers": {
                    "default": 100,
                    "description": "Maximum servers in the pool",
                    "format": "int32",
                    "type": "integer"
                  },
                  "minServers": {
                    "default": 10,
                    "description": "Minimum servers to maintain",
                    "format": "int32",
                    "type": "integer"
                  },
                  "nodeSelector": {
                    "additionalProperties": {
                      "type": "string"
                    },
                    "description": "Node selector",
                    "nullable": true,
                    "type": "object"
                  },
                  "priorityClassName": {
                    "description": "Priority class name",
                    "nullable": true,
                    "type": "string"
                  },
                  "scaleDownDelay": {
                    "default": 60,
                    "description": "Delay before scaling down (seconds)",
                    "format": "int32",
                    "type": "integer"
                  },
                  "serverDefaults": {
                    "default": {
                      "image": null,
                      "livenessProbe": null,
                      "networking": null,
                      "resources": null,
                      "securityContext": null
                    },
                    "description": "Server defaults",
                    "properties": {
                      "image": {
                        "description": "Default image",
                        "nullable": true,
                        "type": "string"
                      },
                      "livenessProbe": {
                        "description": "Default liveness probe",
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
                      "networking": {
                        "description": "Default networking",
                        "nullable": true,
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
                      "resources": {
                        "description": "Default resources",
                        "nullable": true,
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
                        "description": "Default security context",
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
                      }
                    },
                    "type": "object"
                  },
                  "serverTimeout": {
                    "default": 300,
                    "description": "Server idle timeout in seconds",
                    "format": "int32",
                    "type": "integer"
                  },
                  "tolerations": {
                    "default": [],
                    "description": "Pod tolerations",
                    "items": {
                      "description": "Pod toleration",
                      "properties": {
                        "effect": {
                          "description": "Toleration effect",
                          "nullable": true,
                          "type": "string"
                        },
                        "key": {
                          "description": "Toleration key",
                          "nullable": true,
                          "type": "string"
                        },
                        "operator": {
                          "description": "Toleration operator",
                          "nullable": true,
                          "type": "string"
                        },
                        "value": {
                          "description": "Toleration value",
                          "nullable": true,
                          "type": "string"
                        }
                      },
                      "type": "object"
                    },
                    "type": "array"
                  },
                  "upgradeStrategy": {
                    "default": {
                      "maxSurge": null,
                      "maxUnavailable": null,
                      "type": ""
                    },
                    "description": "Upgrade strategy",
                    "properties": {
                      "maxSurge": {
                        "description": "Max surge",
                        "nullable": true,
                        "type": "string"
                      },
                      "maxUnavailable": {
                        "description": "Max unavailable",
                        "nullable": true,
                        "type": "string"
                      },
                      "type": {
                        "default": "RollingUpdate",
                        "description": "Strategy type",
                        "type": "string"
                      }
                    },
                    "type": "object"
                  }
                },
                "type": "object"
              },
              "status": {
                "description": "McpPool status",
                "nullable": true,
                "properties": {
                  "availableServers": {
                    "description": "Available servers count",
                    "format": "int32",
                    "nullable": true,
                    "type": "integer"
                  },
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
                  "inUseServers": {
                    "description": "In use servers count",
                    "format": "int32",
                    "nullable": true,
                    "type": "integer"
                  },
                  "lastScaleDownTime": {
                    "description": "Last scale down time",
                    "format": "date-time",
                    "nullable": true,
                    "type": "string"
                  },
                  "lastScaleUpTime": {
                    "description": "Last scale up time",
                    "format": "date-time",
                    "nullable": true,
                    "type": "string"
                  },
                  "metrics": {
                    "description": "Pool metrics",
                    "nullable": true,
                    "properties": {
                      "activeConnections": {
                        "description": "Active connections",
                        "format": "int32",
                        "nullable": true,
                        "type": "integer"
                      },
                      "averageCpuUtilization": {
                        "description": "Average CPU utilization",
                        "nullable": true,
                        "type": "string"
                      },
                      "averageMemoryUtilization": {
                        "description": "Average memory utilization",
                        "nullable": true,
                        "type": "string"
                      },
                      "totalRequests": {
                        "description": "Total requests",
                        "format": "int64",
                        "nullable": true,
                        "type": "integer"
                      }
                    },
                    "type": "object"
                  },
                  "pendingServers": {
                    "description": "Pending servers count",
                    "format": "int32",
                    "nullable": true,
                    "type": "integer"
                  }
                },
                "type": "object"
              }
            },
            "required": [
              "spec"
            ],
            "title": "MCPPool",
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
"#;
