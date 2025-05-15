use crate::{Error, Result};
use kube::{Client, Config};
use serde_yml::from_str;
use std::{fs::read_to_string, path::PathBuf};

pub async fn get_kube_client(kubeconfig: Option<PathBuf>) -> Result<Client, Error> {
    let kubeconfig = match &kubeconfig {
        Some(path) => {
            let kubeconfig = read_to_string(path).map_err(Error::KubeconfigPathNotExists)?;
            let kubeconfig = from_str(&kubeconfig).map_err(Error::KubeConfigParseError)?;
            Config::from_custom_kubeconfig(kubeconfig, &Default::default())
                .await
                .map_err(Error::KubeconfigError)?
        }

        // --- If no kubeconfig path is provided, use the in-cluster config.
        None => Config::incluster().map_err(Error::from)?,
    };

    // --- Create a kube client from the kubeconfig.
    let client = Client::try_from(kubeconfig).map_err(Error::from)?;
    Ok(client)
}
