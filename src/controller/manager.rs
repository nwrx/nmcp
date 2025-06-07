use super::MCP_SERVER_OPERATOR_MANAGER;
use crate::utils::{Error, Result};
use crate::{MCPPool, MCPServer, ResultExt};
use k8s_openapi::NamespaceResourceScope;
use kube::api::{Api, ListParams, ObjectMeta, Patch, PatchParams, PostParams};
use kube::core::object::{HasSpec, HasStatus};
use kube::{Client, Resource, ResourceExt};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::default::Default;
use std::fmt::Debug;
use std::future::Future;

pub trait ResourceManager
where
    Self: Resource<Scope = NamespaceResourceScope>
        + Send
        + Sync
        + Serialize
        + Clone
        + Debug
        + DeserializeOwned
        + HasSpec
        + HasStatus,
    <Self as Resource>::DynamicType: Default,
    <Self as HasSpec>::Spec: Send + Sync + Serialize + Debug,
    <Self as HasStatus>::Status: Send + Sync + Serialize + Default + Clone,
{
    /// Create a new instance of the resource with the given name and spec.
    fn new(name: &str, spec: Self::Spec) -> Self;

    /// Create the specific resource in the Kubernetes cluster.
    fn apply(&self, client: &Client) -> impl Future<Output = Result<Self>> + Send {
        async {
            let post_params = PostParams {
                field_manager: Some(MCP_SERVER_OPERATOR_MANAGER.to_string()),
                ..Default::default()
            };
            Api::<Self>::namespaced(client.clone(), client.default_namespace())
                .create(&post_params, self)
                .await
                .map_err(Error::from)
        }
    }

    /// Get the specific resource from the Kubernetes cluster.
    #[tracing::instrument(name = "GetResource", skip(client))]
    fn get_by_name(client: &Client, name: &str) -> impl Future<Output = Result<Self>> + Send {
        async move {
            Api::<Self>::namespaced(client.clone(), client.default_namespace())
                .get(name)
                .await
                .map_err(Error::from)
                .with_message(format!(
                    "Failed to get '{}' resource with name '{}' from namespace '{}'.",
                    std::any::type_name::<Self>()
                        .split("::")
                        .last()
                        .unwrap_or("Unknown"),
                    name,
                    client.default_namespace()
                ))
        }
    }

    /// Get the status of the specific resource from the Kubernetes cluster.
    fn get_status(&self, client: &Client) -> impl Future<Output = Result<Self::Status>> + Send {
        async {
            let statut = Api::<Self>::namespaced(client.clone(), client.default_namespace())
                .get_status(&self.name_any())
                .await
                .map_err(Error::from)?
                .status()
                .cloned()
                .unwrap_or_default();
            Ok(statut)
        }
    }

    /// Check if the specific resource exists in the Kubernetes cluster.
    fn exists(client: &Client, name: &str) -> impl Future<Output = Result<bool>> + Send {
        async {
            match Api::<Self>::namespaced(client.clone(), client.default_namespace())
                .get_metadata(name)
                .await
            {
                Ok(_) => Ok(true),
                Err(kube::Error::Api(e)) if e.code == 404 => Ok(false),
                Err(e) => Err(Error::from(e)),
            }
        }
    }

    /// Search for resources based on a label selector.
    fn search(
        client: &Client,
        list_params: Option<ListParams>,
    ) -> impl Future<Output = Result<Vec<Self>>> + Send {
        async {
            let params = list_params.unwrap_or_default();
            Api::<Self>::namespaced(client.clone(), client.default_namespace())
                .list(&params)
                .await
                .map_err(Error::from)
                .map(|list| list.items)
        }
    }

    /// Patch the specific resource in the Kubernetes cluster.
    fn patch(
        &self,
        client: &Client,
        spec: Self::Spec,
    ) -> impl Future<Output = Result<Self>> + Send {
        async move {
            let patch = serde_json::json!({
                "apiVersion": "nmcp.nwrx.io/v1",
                "kind": "MCPServer",
                "spec": spec
            });
            Api::<Self>::namespaced(client.clone(), client.default_namespace())
                .patch(
                    &self.name_any(),
                    &PatchParams::apply(MCP_SERVER_OPERATOR_MANAGER),
                    &Patch::Apply(patch),
                )
                .await
                .map_err(Error::from)
        }
    }

    /// Patch the status of the specific resource in the Kubernetes cluster.
    fn patch_status(
        &self,
        client: &Client,
        status: Self::Status,
    ) -> impl Future<Output = Result<Self>> + Send {
        async move {
            Api::<Self>::namespaced(client.clone(), client.default_namespace())
                .patch_status(
                    &self.name_any(),
                    &PatchParams::apply(MCP_SERVER_OPERATOR_MANAGER),
                    &Patch::Merge(&serde_json::json!({ "status": status })),
                )
                .await
                .map_err(Error::from)
        }
    }

    /// Refresh the specific resource from the Kubernetes cluster.
    fn refresh(&self, client: &Client) -> impl Future<Output = Result<Self>> + Send {
        async {
            Api::<Self>::namespaced(client.clone(), client.default_namespace())
                .get_status(&self.name_any())
                .await
                .map_err(Error::from)
        }
    }

    /// Delete the specific resource from the Kubernetes cluster.
    fn delete(&self, client: &Client) -> impl Future<Output = Result<()>> + Send {
        async {
            match Api::<Self>::namespaced(client.clone(), client.default_namespace())
                .delete(&self.name_any(), &Default::default())
                .await
            {
                Ok(..) => Ok(()),
                Err(kube::Error::Api(e)) if e.code == 404 => Ok(()),
                Err(error) => Err(Error::from(error)),
            }
        }
    }
}

impl ResourceManager for MCPPool {
    fn new(name: &str, spec: Self::Spec) -> Self {
        Self {
            metadata: ObjectMeta {
                name: Some(name.to_string()),
                ..Default::default()
            },
            spec,
            status: Default::default(),
        }
    }
}

impl ResourceManager for MCPServer {
    fn new(name: &str, spec: Self::Spec) -> Self {
        Self {
            metadata: ObjectMeta {
                name: Some(name.to_string()),
                ..Default::default()
            },
            spec,
            status: Default::default(),
        }
    }
}
