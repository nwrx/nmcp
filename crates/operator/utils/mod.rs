mod error;
mod get_kube_client;

pub use error::{Error, Result};
pub use get_kube_client::get_kube_client;
