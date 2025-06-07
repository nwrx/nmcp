use crate::{Error, ErrorInner, Result};
use kube::{config, Client, Config};
use std::{fs::read_to_string, path::PathBuf, str::FromStr};

#[derive(Debug, Clone, Default)]
pub enum Kubeconfig {
    /// Use the in-cluster config.
    #[default]
    InCluster,

    /// Path to the kubeconfig file.
    Path(PathBuf),

    /// Kubeconfig
    Kubeconfig(Box<config::Kubeconfig>),
}

impl FromStr for Kubeconfig {
    type Err = Error;
    fn from_str(config: &str) -> Result<Self> {
        if config.is_empty() {
            Ok(Self::InCluster)

        // --- Is a path to a file.
        } else if config.starts_with("/") {
            match PathBuf::from(config).canonicalize() {
                Ok(path) => Ok(Self::Path(path)),
                Err(e) => Err(e.into()),
            }

        // --- Otherwise, return an error.
        } else {
            Err(Error::generic("Kubeconfig path is not valid. Please provide a valid path or an empty string for in-cluster config."))
        }
    }
}

impl From<PathBuf> for Kubeconfig {
    fn from(path: PathBuf) -> Self {
        Self::Path(path)
    }
}

impl From<config::Kubeconfig> for Kubeconfig {
    fn from(kubeconfig: config::Kubeconfig) -> Self {
        let kubeconfig = Box::new(kubeconfig);
        Self::Kubeconfig(kubeconfig)
    }
}

impl Kubeconfig {
    #[cfg(test)]
    pub async fn from_container(
        container: &testcontainers::ContainerAsync<testcontainers_modules::k3s::K3s>,
    ) -> Result<Self> {
        // --- Get the kubeconfig path from the K3s instance.
        let config = container
            .image()
            .read_kube_config()
            .expect("Failed to read kubeconfig");

        // --- Get the kube client.
        let port = container
            .get_host_port_ipv4(testcontainers_modules::k3s::KUBE_SECURE_PORT)
            .await
            .expect("Failed to get host port");

        // --- Create the kubeconfig from the YAML string.
        let mut kubeconfig =
            config::Kubeconfig::from_yaml(&config).expect("Failed to create kube config");

        // --- Update the kubeconfig with the host port from the Testcontainers instance
        // --- and use the static port provided by the `testcontainers-modules::k3s` module.
        kubeconfig.clusters.iter_mut().for_each(|cluster| {
            if let Some(cluster) = cluster.cluster.as_mut() {
                if let Some(server) = cluster.server.as_mut() {
                    *server = format!("https://127.0.0.1:{port}");
                }

                // --- Ignore TLS verification for the testcontainers instance.
                // --- This is required for the testcontainers instance to work.
                cluster.insecure_skip_tls_verify = Some(true);
            }
        });

        Ok(Self::Kubeconfig(Box::new(kubeconfig)))
    }
}

impl From<Kubeconfig> for config::Kubeconfig {
    fn from(kubeconfig: Kubeconfig) -> Self {
        match kubeconfig {
            Kubeconfig::Path(path) => {
                let kubeconfig = read_to_string(path).unwrap_or_default();
                serde_yml::from_str(&kubeconfig).unwrap_or_default()
            }
            Kubeconfig::Kubeconfig(kubeconfig) => *kubeconfig,
            Kubeconfig::InCluster => Self::default(),
        }
    }
}

pub async fn get_kube_client(kubeconfig: Kubeconfig) -> Result<Client> {
    let config = match &kubeconfig {
        Kubeconfig::Path(path) => {
            let kubeconfig = read_to_string(path).map_err(Error::from)?;
            let kubeconfig = serde_yml::from_str(&kubeconfig).map_err(ErrorInner::from)?;
            Config::from_custom_kubeconfig(kubeconfig, &Default::default())
                .await
                .map_err(ErrorInner::KubeconfigError)?
        }

        // --- If a kubeconfig instance is provided, use it.
        Kubeconfig::Kubeconfig(kubeconfig) => {
            let kubeconfig = *kubeconfig.clone();
            Config::from_custom_kubeconfig(kubeconfig, &Default::default())
                .await
                .map_err(ErrorInner::KubeconfigError)?
        }

        // --- If no kubeconfig path is provided, use the in-cluster config.
        Kubeconfig::InCluster => Config::incluster().map_err(ErrorInner::from)?,
    };

    // --- Create a kube client from the kubeconfig.
    let client = Client::try_from(config)?;
    Ok(client)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests the creation of a kubeconfig from a path.

    #[test]
    fn test_kubeconfig_from_path_buf() {
        let path = std::env::temp_dir().join("kubeconfig.yaml");
        std::fs::write(&path, "apiVersion: v1").unwrap();
        let result = Kubeconfig::from(path.clone());
        match result {
            Kubeconfig::Path(p) => assert_eq!(p, path),
            _ => panic!("Expected Kubeconfig::Path"),
        }
    }

    #[test]
    fn test_kubeconfig_from_path_str() {
        let path = std::env::temp_dir().join("kubeconfig.yaml");
        let result = Kubeconfig::from_str(path.to_str().unwrap());
        match result {
            Ok(Kubeconfig::Path(p)) => assert_eq!(p, path),
            _ => panic!("Expected Kubeconfig::Path"),
        }
    }

    #[test]
    fn test_kubeconfig_from_incluster() {
        let result = Kubeconfig::from_str("");
        match result {
            Ok(Kubeconfig::InCluster) => {}
            _ => panic!("Expected Kubeconfig::InCluster"),
        }
    }

    // #[test]
    // fn test_kubeconfig_from_invalid_path() {
    //     let result = Kubeconfig::from_str("/invalid/path/to/kubeconfig.yaml");
    //     match result {
    //         Err(Error::KubeconfigPathNotExists(_)) => {}
    //         _ => panic!("Expected KubeconfigPathNotExists error"),
    //     }
    // }

    // #[test]
    // fn test_kubeconfig_from_invalid_string() {
    //     let result = Kubeconfig::from_str("invalid_string");
    //     match result {
    //         Err(Error::KubeconfigPathNotExists(_)) => {}
    //         _ => panic!("Expected KubeconfigPathNotExists error"),
    //     }
    // }

    #[test]
    fn test_kubeconfig_from_kubeconfig() {
        let kubeconfig = config::Kubeconfig::default();
        let result = Kubeconfig::from(kubeconfig.clone());
        match result {
            Kubeconfig::Kubeconfig(k1) => {
                let k1 = serde_json::json!(k1).to_string();
                let k2 = serde_json::json!(kubeconfig).to_string();
                assert_eq!(k1, k2);
            }
            _ => panic!("Expected Kubeconfig::Kubeconfig"),
        }
    }

    //////////////////////////////////////////////////////////////

    fn create_kubeconfig() -> config::Kubeconfig {
        config::Kubeconfig {
            contexts: vec![config::NamedContext {
                name: "test".to_string(),
                context: Some(config::Context {
                    cluster: "test".to_string(),
                    ..Default::default()
                }),
            }],
            clusters: vec![config::NamedCluster {
                name: "test".to_string(),
                cluster: Some(config::Cluster {
                    server: Some("https://localhost:6443".to_string()),
                    ..Default::default()
                }),
            }],
            current_context: Some("test".to_string()),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_get_kube_client_from_kubeconfig() {
        let kubeconfig = create_kubeconfig();
        let kubeconfig = Kubeconfig::from(kubeconfig);
        let _ = get_kube_client(kubeconfig)
            .await
            .expect("Failed to create kube client");
    }

    #[tokio::test]
    async fn test_get_kube_client_from_path() {
        let path = std::env::temp_dir().join("kubeconfig.yaml");
        let kubeconfig = create_kubeconfig();
        let kubeconfig = serde_yml::to_string(&kubeconfig).unwrap();
        std::fs::write(&path, kubeconfig).unwrap();
        let kubeconfig = Kubeconfig::from(path);
        let _ = get_kube_client(kubeconfig)
            .await
            .expect("Failed to create kube client");
    }
}
