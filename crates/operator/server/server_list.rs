use super::ServerState;
use crate::MCPServerSpec;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Query parameters for server list endpoint
#[derive(Deserialize)]
pub struct ServerListQuery {
    pool: Option<String>,
}

/// Server list response
#[derive(Serialize)]
struct ServerListResponse {
    items: Vec<MCPServerSpec>,
}

/// Handler for GET /api/v1/servers
pub async fn server_list(
    State(state): State<Arc<ServerState>>,
    Query(query): Query<ServerListQuery>,
) -> Response {
    let servers = state.controller().list_servers().await;
    let response = match servers {
        Ok(servers) => {
            let filtered_servers = if let Some(pool_name) = query.pool {
                servers
                    .into_iter()
                    .filter(|s| s.spec.pool == pool_name)
                    .map(|s| s.spec)
                    .collect()
            } else {
                servers.into_iter().map(|s| s.spec).collect()
            };

            ServerListResponse {
                items: filtered_servers,
            }
        }
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch servers").into_response();
        }
    };

    (StatusCode::OK, Json(response.items)).into_response()
}
