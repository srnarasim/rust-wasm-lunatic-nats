//! Rust/WASM application using Lunatic and NATS for distributed agent-based systems

pub mod agent;
pub mod memory;
pub mod nats_comm;
pub mod supervisor;
pub mod wasm_nats;

// Re-export commonly used items
pub use agent::{Agent, AgentState, AgentId, Message, StateAction};
pub use memory::MemoryBackend;
pub use nats_comm::{NatsConfig, NatsConnection};
pub use supervisor::{
    AgentConfig, MemoryBackendType, AgentProcess, AgentSupervisor,
    spawn_agent_supervisor, spawn_single_agent, 
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
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
