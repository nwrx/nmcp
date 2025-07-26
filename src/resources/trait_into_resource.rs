use crate::{Error, Result, NMCP_OPERATOR};
use k8s_openapi::NamespaceResourceScope;
use kube::api::{Patch, PatchParams};
use kube::core::object::{HasSpec, HasStatus};
use kube::{Api, Client, Resource, ResourceExt};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;
use std::future::Future;

pub trait IntoResource<U>
where
    U: Resource<Scope = NamespaceResourceScope>
        + Debug
        + Clone
        + Send
        + Sync
        + Serialize
        + DeserializeOwned,
    <U as Resource>::DynamicType: Default,
    Self: Send + Sync + Resource + Sized + HasSpec + HasStatus,
{
    /// Transform the current resource into a specific Kubernetes resource type.
    fn resource(&self) -> U;

    /// Generate the name of the specific resource in the Kubernetes cluster.
    fn resource_name(&self) -> String;

    // Create the specific resource in the Kubernetes cluster.
    fn patch_resource(&self, client: &Client) -> impl Future<Output = Result<U>> + Send {
        async {
            Api::<U>::namespaced(client.clone(), client.default_namespace())
                .patch(
                    &self.resource_name(),
                    &PatchParams::apply(NMCP_OPERATOR),
                    &Patch::Apply(self.resource()),
                )
                .await
                .map_err(Error::from)
        }
    }

    /// Create the specific resource in the Kubernetes cluster.
    fn delete_resource(&self, client: &Client) -> impl Future<Output = Result<()>> + Send {
        async {
            match Api::<U>::namespaced(client.clone(), client.default_namespace())
                .delete(&self.resource_name(), &Default::default())
                .await
            {
                Ok(..) => Ok(()),
                Err(kube::Error::Api(e)) if e.code == 404 => Ok(()),
                Err(error) => Err(Error::from(error)),
            }
        }
    }

    /// Get the specific resource from the Kubernetes cluster.
    fn get_resource(&self, client: &Client) -> impl Future<Output = Result<U>> + Send {
        async {
            Api::<U>::namespaced(client.clone(), client.default_namespace())
                .get(&self.resource_name())
                .await
                .map_err(Error::from)
        }
    }

    /// Check if the specific resource exists in the Kubernetes cluster.
    fn resource_exists(&self, client: &Client) -> impl Future<Output = Result<bool>> + Send {
        async {
            let namespace = self.namespace().unwrap_or("default".to_owned());
            match Api::<U>::namespaced(client.clone(), &namespace)
                .get(&self.resource_name())
                .await
            {
                Ok(_) => Ok(true),
                Err(kube::Error::Api(e)) if e.code == 404 => Ok(false),
                Err(e) => Err(Error::from(e)),
            }
        }
    }
}
