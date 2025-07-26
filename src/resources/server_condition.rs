use crate::Error;
use chrono::Utc;
use k8s_openapi::apimachinery::pkg::apis::meta::v1;
use std::fmt::{Display, Formatter};

/// The reason why a server is stale
#[derive(Debug, Copy, Clone)]
pub enum MCPServerStaleReason {
    /// The server is stale because it has been requested to stop
    ManualShutdown,
    /// The server is stale because it has been idle for too long
    IdleTimeout,
    /// The server is stale because it has been running for too long
    MaxUptimeExceeded,
    /// The server is stale because the configuration has changed
    Outdated,
    /// The server is not stale and is still running
    NotStale,
}

impl Display for MCPServerStaleReason {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

// The reason why a server is requested
#[derive(Debug, Copy, Clone)]
pub enum MCPServerRequestedReason {
    /// A connection was made to the server
    Connection,
}

impl Display for MCPServerRequestedReason {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

/// The various states of a `PodScheduled` reasons.
#[derive(Debug, Clone)]
pub enum MCPServerPodScheduledReason {
    /// The pod scheduling failed
    Failed(Error),
    /// The pod succeeded, meaning it was terminated successfully.
    Succeeded,
    /// The pod has been scheduled successfully
    Scheduled,
    /// The pod is terminating
    Terminating,
}

impl Display for MCPServerPodScheduledReason {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Failed(_) => write!(f, "Failed"),
            Self::Succeeded => write!(f, "Succeeded"),
            Self::Scheduled => write!(f, "Scheduled"),
            Self::Terminating => write!(f, "Terminating"),
        }
    }
}

/// `MCPServerConditionType` follows Kubernetes condition pattern
/// Each condition has a type that represents a specific aspect of the resource's state
#[derive(Debug, Clone)]
pub enum MCPServerCondition {
    /// The server has been requested to start.
    Requested(MCPServerRequestedReason),
    /// The server is stale and needs to be stopped.
    Stale(MCPServerStaleReason),

    /// Pod resource has been created
    PodScheduled(MCPServerPodScheduledReason),
    /// Pod containers are up & ready
    PodReady(Option<Error>),

    /// Service resource has been created
    ServiceCreated(Option<Error>),
    /// Service endpoints are ready
    ServiceReady(Option<Error>),

    /// Reconciliation is in‐flight (starting or stopping)
    Progressing(Option<Error>),
}

impl Display for MCPServerCondition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Requested(_) => write!(f, "Requested"),
            Self::Stale(_) => write!(f, "Stale"),
            Self::PodScheduled(_) => write!(f, "PodScheduled"),
            Self::PodReady(_) => write!(f, "PodReady"),
            Self::ServiceCreated(_) => write!(f, "ServiceCreated"),
            Self::ServiceReady(_) => write!(f, "ServiceReady"),
            Self::Progressing(_) => write!(f, "Progressing"),
        }
    }
}

impl From<MCPServerCondition> for v1::Condition {
    fn from(condition: MCPServerCondition) -> Self {
        match &condition {
            MCPServerCondition::Requested(reason) => Self {
                type_: condition.to_string(),
                reason: reason.to_string(),
                observed_generation: None,
                last_transition_time: v1::Time(Utc::now()),
                status: "True".to_string(),
                message: match reason {
                    MCPServerRequestedReason::Connection => {
                        "Server has been requested to start due to a connection".to_string()
                    }
                },
            },
            MCPServerCondition::Stale(reason) => Self {
                type_: condition.to_string(),
                reason: reason.to_string(),
                observed_generation: None,
                last_transition_time: v1::Time(Utc::now()),
                status: match reason {
                    MCPServerStaleReason::ManualShutdown
                    | MCPServerStaleReason::IdleTimeout
                    | MCPServerStaleReason::MaxUptimeExceeded
                    | MCPServerStaleReason::Outdated => "True",
                    MCPServerStaleReason::NotStale => "False",
                }
                .to_owned(),
                message: match reason {
                    MCPServerStaleReason::ManualShutdown => {
                        "Server is stale due to manual shutdown".to_string()
                    }
                    MCPServerStaleReason::IdleTimeout => {
                        "Server is stale due to idle timeout".to_string()
                    }
                    MCPServerStaleReason::MaxUptimeExceeded => {
                        "Server is stale due to max uptime exceeded".to_string()
                    }
                    MCPServerStaleReason::Outdated => {
                        "Server is stale due to outdated configuration".to_string()
                    }
                    MCPServerStaleReason::NotStale => "Server is not stale".to_string(),
                },
            },
            MCPServerCondition::PodScheduled(state) => Self {
                type_: condition.to_string(),
                reason: condition.to_string(),
                observed_generation: None,
                last_transition_time: v1::Time(Utc::now()),
                status: match state {
                    MCPServerPodScheduledReason::Failed(_) => "False",
                    MCPServerPodScheduledReason::Succeeded => "True",
                    MCPServerPodScheduledReason::Scheduled => "True",
                    MCPServerPodScheduledReason::Terminating => "False",
                }
                .to_owned(),
                message: match state {
                    MCPServerPodScheduledReason::Failed(error) => error.to_string(),
                    MCPServerPodScheduledReason::Succeeded => {
                        "Pod has been terminated successfully".to_string()
                    }
                    MCPServerPodScheduledReason::Scheduled => {
                        "Pod has been scheduled successfully".to_string()
                    }
                    MCPServerPodScheduledReason::Terminating => "Pod is terminating".to_string(),
                },
            },
            MCPServerCondition::PodReady(error) => Self {
                type_: condition.to_string(),
                reason: condition.to_string(),
                observed_generation: None,
                last_transition_time: v1::Time(Utc::now()),
                status: if error.is_some() { "False" } else { "True" }.to_string(),
                message: error
                    .clone()
                    .map(|e| e.to_string())
                    .unwrap_or_else(|| "Pod is ready".to_string()),
            },
            MCPServerCondition::ServiceCreated(error) => Self {
                type_: condition.to_string(),
                reason: condition.to_string(),
                observed_generation: None,
                last_transition_time: v1::Time(Utc::now()),
                status: if error.is_some() { "False" } else { "True" }.to_string(),
                message: error
                    .clone()
                    .map(|e| e.to_string())
                    .unwrap_or_else(|| "Service has been created".to_string()),
            },
            MCPServerCondition::ServiceReady(error) => Self {
                type_: condition.to_string(),
                reason: condition.to_string(),
                observed_generation: None,
                last_transition_time: v1::Time(Utc::now()),
                status: if error.is_some() { "False" } else { "True" }.to_string(),
                message: error
                    .clone()
                    .map(|e| e.to_string())
                    .unwrap_or_else(|| "Service is ready".to_string()),
            },
            MCPServerCondition::Progressing(error) => Self {
                type_: condition.to_string(),
                reason: condition.to_string(),
                observed_generation: None,
                last_transition_time: v1::Time(Utc::now()),
                status: if error.is_some() { "False" } else { "True" }.to_string(),
                message: error
                    .clone()
                    .map(|e| e.to_string())
                    .unwrap_or_else(|| "Reconciliation is in progress".to_string()),
            },
        }
    }
}
