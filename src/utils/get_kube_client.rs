use crate::{Error, Result};
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
            Ok(Kubeconfig::InCluster)

        // --- Is a path to a file.
        } else if config.starts_with("/") {
            match PathBuf::from(config).canonicalize() {
                Ok(path) => Ok(Kubeconfig::Path(path)),
                Err(e) => Err(Error::KubeconfigPathNotExists(e)),
            }

        // --- Otherwise, return an error.
        } else {
            Err(Error::KubeconfigPathNotExists(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Kubeconfig path not found",
            )))
        }
    }
}

impl From<PathBuf> for Kubeconfig {
    fn from(path: PathBuf) -> Self {
        Kubeconfig::Path(path)
    }
}

impl From<&str> for Kubeconfig {
    fn from(path: &str) -> Self {
        Kubeconfig::from_str(path).unwrap_or(Kubeconfig::InCluster)
    }
}

impl From<config::Kubeconfig> for Kubeconfig {
    fn from(kubeconfig: config::Kubeconfig) -> Self {
        let kubeconfig = Box::new(kubeconfig);
        Kubeconfig::Kubeconfig(kubeconfig)
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
            kube::config::Kubeconfig::from_yaml(&config).expect("Failed to create kube config");

        // --- Update the kubeconfig with the host port from the Testcontainers instance.
        kubeconfig.clusters.iter_mut().for_each(|cluster| {
            if let Some(cluster) = cluster.cluster.as_mut() {
                if let Some(server) = cluster.server.as_mut() {
                    *server = format!("https://127.0.0.1:{port}");
                }
            }
        });

        Ok(Kubeconfig::Kubeconfig(Box::new(kubeconfig)))
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
            Kubeconfig::InCluster => config::Kubeconfig::default(),
        }
    }
}

pub async fn get_kube_client(kubeconfig: Kubeconfig) -> Result<Client> {
    let config = match &kubeconfig {
        Kubeconfig::Path(path) => {
            let kubeconfig = read_to_string(path).map_err(Error::KubeconfigPathNotExists)?;
            let kubeconfig = serde_yml::from_str(&kubeconfig).map_err(Error::from)?;
            Config::from_custom_kubeconfig(kubeconfig, &Default::default())
                .await
                .map_err(Error::KubeconfigError)?
        }

        // --- If a kubeconfig instance is provided, use it.
        Kubeconfig::Kubeconfig(kubeconfig) => {
            let kubeconfig = *kubeconfig.clone();
            Config::from_custom_kubeconfig(kubeconfig, &Default::default())
                .await
                .map_err(Error::KubeconfigError)?
        }

        // --- If no kubeconfig path is provided, use the in-cluster config.
        Kubeconfig::InCluster => Config::incluster().map_err(Error::from)?,
    };

    // --- Create a kube client from the kubeconfig.
    let client = Client::try_from(config).map_err(Error::from)?;
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

    #[test]
    fn test_kubeconfig_from_invalid_path() {
        let result = Kubeconfig::from_str("/invalid/path/to/kubeconfig.yaml");
        match result {
            Err(Error::KubeconfigPathNotExists(_)) => {}
            _ => panic!("Expected KubeconfigPathNotExists error"),
        }
    }

    #[test]
    fn test_kubeconfig_from_invalid_string() {
        let result = Kubeconfig::from_str("invalid_string");
        match result {
            Err(Error::KubeconfigPathNotExists(_)) => {}
            _ => panic!("Expected KubeconfigPathNotExists error"),
        }
    }

    #[test]
    fn test_kubeconfig_from_kubeconfig() {
        let kubeconfig = kube::config::Kubeconfig::default();
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

    fn create_kubeconfig() -> kube::config::Kubeconfig {
        kube::config::Kubeconfig {
            contexts: vec![kube::config::NamedContext {
                name: "test".to_string(),
                context: Some(kube::config::Context {
                    cluster: "test".to_string(),
                    ..Default::default()
                }),
            }],
            clusters: vec![kube::config::NamedCluster {
                name: "test".to_string(),
                cluster: Some(kube::config::Cluster {
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
        get_kube_client(kubeconfig)
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
        get_kube_client(kubeconfig)
            .await
            .expect("Failed to create kube client");
    }
}
