//! Rust/WASM application using Lunatic and NATS for distributed agent-based systems

pub mod agent;
pub mod llm_client;
pub mod memory;
pub mod nats_comm;
pub mod supervisor;
pub mod wasm_nats;

// Re-export commonly used items
pub use agent::{Agent, AgentState, AgentId, Message, StateAction};
pub use llm_client::{LLMClient, LLMProvider, LLMRequest, LLMResponse, WorkflowStep, create_llm_client};
pub use memory::MemoryBackend;
pub use nats_comm::{NatsConfig, NatsConnection};
pub use supervisor::{
    AgentConfig, MemoryBackendType, AgentType, AgentProcess, AgentSupervisor,
    spawn_agent_supervisor, spawn_single_agent, spawn_llm_enabled_agent,
    send_message_to_agent, send_state_action_to_agent,
    get_agent_state, shutdown_agent, GetAgentState, Shutdown
};
pub use wasm_nats::{WasmNatsConfig, WasmNatsConnection, WasmConnectionStats, WasmNatsPublisher};

/// Common result type for the library
pub type Result<T> = std::result::Result<T, Error>;

/// Common error type for the library
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("NATS error: {0}")]
    Nats(String),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Custom error: {0}")]
    Custom(String),

    // New LLM-related errors
    #[error("LLM provider error: {0}")]
    LLMProvider(String),
    
    #[error("LLM API timeout after {timeout}s")]
    LLMTimeout { timeout: u64 },
    
    #[error("LLM rate limit exceeded: {0}")]
    LLMRateLimit(String),
    
    #[error("Invalid LLM response format: {0}")]
    LLMResponseFormat(String),
    
    #[error("Workflow validation error: {0}")]
    WorkflowValidation(String),
}

// Enhanced error handling methods
impl Error {
    pub fn is_retryable(&self) -> bool {
        matches!(self, 
            Error::LLMTimeout { .. } | 
            Error::LLMRateLimit(_) |
            Error::Nats(_)
        )
    }

    pub fn retry_delay_ms(&self) -> u64 {
        match self {
            Error::LLMTimeout { .. } => 1000,
            Error::LLMRateLimit(_) => 5000,
            Error::Nats(_) => 500,
            _ => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
