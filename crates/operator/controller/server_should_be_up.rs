use super::Controller;
use crate::{MCPServer, Result};
use chrono::Utc;

impl Controller {
    /// Determine if a server should be up based on pool limits and idle timeout
    ///
    /// This function checks various conditions to determine if the server should be running:
    /// 1. Pool limits (max_servers_limit and max_servers_active)
    /// 2. Idle timeout based on last request time
    ///
    /// # Arguments
    /// * `server` - A reference to the MCPServer object.
    ///
    /// # Returns
    /// * `Result<bool>` - Whether the server should be running or not.
    ///
    /// # Errors
    /// * Returns an error if there's an issue retrieving the pool information.
    ///
    /// # Example
    /// ```
    /// let controller = Controller::new("default".to_string(), kube_client).await?;
    /// let should_be_up = controller.server_should_be_up(&server).await?;
    /// ```
    pub async fn server_should_be_up(&self, server: &MCPServer) -> Result<bool> {
        let pool = self.server_get_pool(server).await?;

        // Check pool limits
        if let Some(pool_status) = &pool.status {
            // If managed_servers_count > max_servers_limit, server should not be up
            if pool_status.managed_servers_count > pool.spec.max_servers_limit {
                return Ok(false);
            }

            // If active_servers_count >= max_servers_active, server should not be up
            if pool_status.active_servers_count >= pool.spec.max_servers_active {
                return Ok(false);
            }
        }

        // Check idle timeout
        if let Some(server_status) = &server.status {
            if server_status.is_running {
                if let Some(last_request) = &server_status.last_request_at {
                    // Get the relevant idle timeout (server's value or pool's default)
                    let idle_timeout = if server.spec.idle_timeout > 0 {
                        server.spec.idle_timeout
                    } else {
                        pool.spec.default_idle_timeout
                    };

                    // If elapsed time is greater than the idle timeout, server should not be up
                    let now = Utc::now();
                    let elapsed = now.signed_duration_since(*last_request);
                    let elapsed_secs = elapsed.num_seconds();
                    if elapsed_secs > idle_timeout as i64 {
                        return Ok(false);
                    }
                }
            }
        }

        // If none of the conditions to be down are met, the server should be up
        Ok(true)
    }
}
