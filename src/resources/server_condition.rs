use crate::Error;
use chrono::Utc;
use k8s_openapi::apimachinery::pkg::apis::meta::v1;
use std::fmt::{Display, Formatter};

/// The reason why a server is stale
#[derive(Debug, Copy, Clone)]
pub enum MCPServerStaleState {
    /// The server is stale because it has been requested to stop
    ManualShutdown,
    /// The server is stale because it has been idle for too long
    IdleTimeout,
    /// The server is stale because it has been running for too long
    UptimeExceeded,
    /// The server is stale because the configuration has changed
    Outdated,
    /// The server is not stale and is still running
    NotStale,
}

impl Display for MCPServerStaleState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

// The reason why a server is requested
#[derive(Debug, Copy, Clone)]
pub enum MCPServerRequestedState {
    /// A connection was made to the server
    Connection,
    /// The server was manually requested to start
    ManualStart,
    /// The server was manually requested to stop
    ManualStop,
    /// The server is not requested to do anything
    Unused,
}

impl Display for MCPServerRequestedState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

/// The various states of a `PodScheduled` reasons.
#[derive(Debug, Clone)]
pub enum MCPServerPodScheduledState {
    /// The pod scheduling failed
    Failed(Error),
    /// The pod succeeded, meaning it was terminated successfully.
    Succeeded,
    /// The pod is terminating
    Terminating,
    /// The pod has been scheduled successfully
    Scheduled,
    /// The pod is running
    Running,
}

impl Display for MCPServerPodScheduledState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Failed(_) => write!(f, "Failed"),
            Self::Succeeded => write!(f, "Succeeded"),
            Self::Scheduled => write!(f, "Scheduled"),
            Self::Terminating => write!(f, "Terminating"),
            Self::Running => write!(f, "Running"),
        }
    }
}

/// `MCPServerConditionType` follows Kubernetes condition pattern
/// Each condition has a type that represents a specific aspect of the resource's state
#[derive(Debug, Clone)]
pub enum MCPServerCondition {
    /// The server has been requested to start.
    Requested(MCPServerRequestedState),
    /// The server is stale and needs to be stopped.
    Stale(MCPServerStaleState),
    /// Pod resource has been created
    PodScheduled(MCPServerPodScheduledState),
    /// Service resource has been created
    ServiceCreated(Option<Error>),
}

impl Display for MCPServerCondition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Requested(_) => write!(f, "Requested"),
            Self::Stale(_) => write!(f, "Stale"),
            Self::PodScheduled(_) => write!(f, "PodScheduled"),
            Self::ServiceCreated(_) => write!(f, "ServiceCreated"),
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
                status: match reason {
                    MCPServerRequestedState::Connection => "True",
                    MCPServerRequestedState::ManualStart => "True",
                    MCPServerRequestedState::ManualStop => "False",
                    MCPServerRequestedState::Unused => "False",
                }
                .to_owned(),
                message: match reason {
                    MCPServerRequestedState::Connection => "Due to a connection".to_string(),
                    MCPServerRequestedState::ManualStart => "Due to manual start".to_string(),
                    MCPServerRequestedState::ManualStop => "Due to manual stop".to_string(),
                    MCPServerRequestedState::Unused => "Not requested to do anything".to_string(),
                },
            },
            MCPServerCondition::Stale(state) => Self {
                type_: condition.to_string(),
                reason: state.to_string(),
                observed_generation: None,
                last_transition_time: v1::Time(Utc::now()),
                status: match state {
                    MCPServerStaleState::ManualShutdown => "True",
                    MCPServerStaleState::IdleTimeout => "True",
                    MCPServerStaleState::UptimeExceeded => "True",
                    MCPServerStaleState::Outdated => "True",
                    MCPServerStaleState::NotStale => "False",
                }
                .to_owned(),
                message: match state {
                    MCPServerStaleState::ManualShutdown => "Due to manual shutdown".to_string(),
                    MCPServerStaleState::IdleTimeout => "Due to idle timeout".to_string(),
                    MCPServerStaleState::UptimeExceeded => "Due to uptime exceeded".to_string(),
                    MCPServerStaleState::Outdated => "Due to outdated configuration".to_string(),
                    MCPServerStaleState::NotStale => "Server is not stale".to_string(),
                },
            },
            MCPServerCondition::PodScheduled(state) => Self {
                type_: condition.to_string(),
                reason: state.to_string(),
                observed_generation: None,
                last_transition_time: v1::Time(Utc::now()),
                status: match state {
                    MCPServerPodScheduledState::Failed(_) => "False",
                    MCPServerPodScheduledState::Succeeded => "False",
                    MCPServerPodScheduledState::Terminating => "False",
                    MCPServerPodScheduledState::Scheduled => "True",
                    MCPServerPodScheduledState::Running => "True",
                }
                .to_owned(),
                message: match state {
                    MCPServerPodScheduledState::Failed(error) => error.to_string(),
                    MCPServerPodScheduledState::Succeeded => "Pod has been terminated".to_string(),
                    MCPServerPodScheduledState::Terminating => "Pod is terminating".to_string(),
                    MCPServerPodScheduledState::Scheduled => "Pod has been scheduled".to_string(),
                    MCPServerPodScheduledState::Running => "Pod is running".to_string(),
                },
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
        }
    }
}
