use kube::{Client, Config, config::{Kubeconfig, KubeConfigOptions}};
use std::fs;
use tracing::info;
use crate::{Error, Result};
use super::Program;

impl Program {
    /// Retrieves a Kubernetes client configured based on the program's settings.
    ///
    /// This method attempts to create a Kubernetes client by either using a user-provided
    /// kubeconfig file or falling back to the default kubeconfig settings. The client is
    /// used to interact with the Kubernetes API.
    ///
    /// # Behavior
    ///
    /// 1. **User-Provided Kubeconfig**:
    ///    - If the `kubeconfig` field in the `Program` struct is set, the method will:
    ///      - Read the kubeconfig file from the specified path.
    ///      - Parse the kubeconfig YAML content.
    ///      - Create a Kubernetes client configuration from the parsed kubeconfig.
    ///      - Instantiate a Kubernetes client using the generated configuration.
    ///
    /// 2. **Default Kubeconfig**:
    ///    - If no kubeconfig is provided, the method will:
    ///      - Use the default kubeconfig settings, which typically look for a kubeconfig
    ///        file in `~/.kube/config` or use the `KUBECONFIG` environment variable.
    ///      - Instantiate a Kubernetes client using the default configuration.
    ///
    /// # Errors
    ///
    /// This method returns a `Result` with the following possible errors:
    /// - If the kubeconfig file cannot be read, a `KubeconfigReadError` is returned with details
    ///   about the failure.
    /// - If the kubeconfig file cannot be parsed, a `KubeconfigParseError` is returned with details
    ///   about the parsing issue.
    /// - If the client configuration cannot be created from the kubeconfig, a `KubeconfigConfigError`
    ///   is returned with details about the failure.
    /// - If the Kubernetes client cannot be instantiated, a `KubeClientCreationError` is returned
    ///   with details about the failure.
    /// - If the default kubeconfig cannot be used, a `KubeconfigError` is returned.
    ///
    /// # Logging
    ///
    /// - Logs an informational message when using a user-provided kubeconfig, including
    ///   the path to the kubeconfig file.
    /// - Logs an informational message when falling back to the default kubeconfig.
    /// - Logs a success message upon successfully connecting to the Kubernetes API.
    ///
    /// # Returns
    ///
    /// - On success, returns an instance of `Client` that can be used to interact with
    ///   the Kubernetes API.
    /// - On failure, returns an error wrapped in a `Result`.
    ///
    pub async fn get_client(&self) -> Result<Client> {

        // --- If the user provided a kubeconfig, read, parse and create a client from it.
        let client = if let Some(kubeconfig_path) = &self.kubeconfig {
            info!("Using provided kubeconfig: {:?}", kubeconfig_path);
            
            // Read kubeconfig from file
            let kubeconfig_yaml = fs::read_to_string(kubeconfig_path)
                .map_err(|e| Error::KubeconfigReadError {
                    path: kubeconfig_path.display().to_string(),
                    error: e,
                })?;
            
            // Parse kubeconfig
            let kubeconfig = Kubeconfig::from_yaml(&kubeconfig_yaml)
                .map_err(|e| Error::KubeconfigParseError {
                    path: kubeconfig_path.display().to_string(),
                    error: e,
                })?;
            
            // Create client from kubeconfig
            let client_config_options = KubeConfigOptions::default();
            let client_config = Config::from_custom_kubeconfig(kubeconfig, &client_config_options).await
                .map_err(|e| Error::KubeconfigConfigError {
                    path: kubeconfig_path.display().to_string(),
                    error: e,
                })?;
            
            Client::try_from(client_config)
                .map_err(|e| Error::KubeClientCreationError {
                    path: kubeconfig_path.display().to_string(),
                    error: e,
                })?

        // --- If no kubeconfig is provided, use the default kubeconfig. By default, this will
        // --- use the kubeconfig in ~/.kube/config or the KUBECONFIG environment variable.
        // --- This is the default behavior of the kube crate.
        } else {
            info!("Using default kubeconfig");
            Client::try_default().await
                .map_err(|e| Error::KubeconfigError {
                    error: e,
                })?
        };
        
        info!("Successfully connected to Kubernetes API");
        Ok(client)
    }
}
