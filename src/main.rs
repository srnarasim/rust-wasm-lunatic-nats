#[cfg(feature = "nats")]
use env_logger;
use log::{info, error, warn};
#[cfg(feature = "nats")]
use std::time::Duration;

// Include the library modules
mod agent;
mod http_client;  // Add missing http_client module
mod llm_client;  
mod memory; 
mod nats_comm;
mod supervisor;
mod wasm_nats;

// Re-export commonly used items
use agent::{AgentId, Message, StateAction};
use nats_comm::{NatsConfig, NatsConnection};
use supervisor::{
    AgentConfig, MemoryBackendType, AgentType,
    spawn_agent_supervisor, spawn_single_agent,
    send_message_to_agent, send_state_action_to_agent,
    get_agent_state, shutdown_agent
};

// Common result type
type Result<T> = std::result::Result<T, Error>;

// Common error type
#[derive(Debug, thiserror::Error)]
enum Error {
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

#[cfg(feature = "nats")]
#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    nats_main().await
}

#[cfg(not(feature = "nats"))]
fn main() -> Result<()> {
    // For WASM builds without tokio/env_logger - use basic sync main
    wasm_main()
}

#[cfg(feature = "nats")]
async fn nats_main() -> Result<()> {
    info!("Starting Rust WASM Lunatic NATS application with supervisor model");

    // Note: For now, we'll demonstrate the supervisor pattern without 
    // the complex Lunatic process spawning, which requires proper WASM compilation
    info!("Demonstrating Lunatic supervisor pattern concepts");

    // Create agent configurations
    let agent_configs = vec![
        AgentConfig {
            id: AgentId("worker_agent_1".to_string()),
            memory_backend_type: MemoryBackendType::InMemory,
            nats_enabled: true,
            llm_enabled: false,
            agent_type: AgentType::Generic,
        },
        AgentConfig {
            id: AgentId("worker_agent_2".to_string()),
            memory_backend_type: MemoryBackendType::InMemory,
            nats_enabled: true,
            llm_enabled: false,
            agent_type: AgentType::Generic,
        },
        AgentConfig {
            id: AgentId("monitor_agent".to_string()),
            memory_backend_type: MemoryBackendType::InMemory,
            nats_enabled: false,
            llm_enabled: false,
            agent_type: AgentType::Generic,
        },
    ];

    info!("Created {} agent configurations", agent_configs.len());

    // In a real Lunatic environment, these would spawn as separate WASM processes
    info!("Note: To run with full Lunatic supervisor model:");
    info!("1. Compile with: cargo build --target=wasm32-wasi");
    info!("2. Run with: lunatic run target/wasm32-wasi/debug/rust-wasm-lunatic-nats.wasm");

    // Demonstrate the API that would be used in Lunatic
    demonstrate_supervisor_api(agent_configs).await?;

    // Create NATS configuration (optional)
    let nats_config = NatsConfig {
        url: std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string()),
        timeout: Duration::from_secs(10),
        max_reconnects: Some(10),
        reconnect_delay: Duration::from_secs(1),
    };

    // Try to connect to NATS (system works without it)
    info!("Attempting to connect to NATS at {}", nats_config.url);
    match NatsConnection::new(nats_config).await {
        Ok(nats_conn) => {
            info!("Connected to NATS successfully");
            demonstrate_nats_integration(nats_conn).await?;
        }
        Err(e) => {
            warn!("Failed to connect to NATS: {}. Continuing without NATS.", e);
        }
    }

    info!("Application completed successfully");
    Ok(())
}

async fn demonstrate_supervisor_api(configs: Vec<AgentConfig>) -> Result<()> {
    info!("=== Demonstrating Supervisor API ===");
    
    for config in &configs {
        info!("Agent config: {} (NATS: {})", 
              config.id.0, config.nats_enabled);
    }

    // Show how the supervisor would be used in Lunatic
    info!("In Lunatic WASM environment:");
    info!("  let supervisor = spawn_agent_supervisor(configs)?;");
    info!("  let agent = spawn_single_agent(config)?;");
    
    // Demonstrate message structures
    let test_message = Message {
        id: "demo_msg_1".to_string(),
        from: AgentId("main".to_string()),
        to: AgentId("worker_agent_1".to_string()),
        payload: serde_json::json!({
            "type": "task",
            "data": "process_data",
            "priority": "high"
        }),
        timestamp: chrono::Utc::now().timestamp() as u64,
    };

    info!("Example message: {:?}", test_message);

    // Demonstrate state actions
    let state_action = StateAction::Store {
        key: "config".to_string(),
        value: serde_json::json!({
            "max_workers": 10,
            "timeout": 30,
            "retry_count": 3
        }),
    };

    info!("Example state action: {:?}", state_action);

    Ok(())
}

async fn demonstrate_nats_integration(nats_conn: NatsConnection) -> Result<()> {
    info!("=== Demonstrating NATS Integration ===");

    // Publish a test message
    let test_message = serde_json::json!({
        "type": "system_startup",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "supervisor": "main_supervisor",
        "agents": ["worker_agent_1", "worker_agent_2", "monitor_agent"]
    });

    if let Err(e) = nats_conn.publish("system.events", test_message.to_string().as_bytes()).await {
        error!("Failed to publish test message: {}", e);
    } else {
        info!("Published system startup message to NATS");
    }

    // Check connection stats
    let stats = nats_conn.get_stats();
    info!("NATS connection stats: sent={}, received={}, reconnects={}", 
          stats.messages_sent, stats.messages_received, stats.reconnects);

    // Demonstrate agent-to-agent messaging pattern
    let agent_message = serde_json::json!({
        "id": format!("msg_{}", chrono::Utc::now().timestamp_nanos()),
        "from": {"0": "worker_agent_1"},
        "to": {"0": "worker_agent_2"},
        "payload": {
            "type": "work_result",
            "data": "completed_task_123",
            "success": true
        },
        "timestamp": chrono::Utc::now().timestamp()
    });

    if let Err(e) = nats_conn.publish("agent.worker_agent_2", 
                                     serde_json::to_string(&agent_message)?.as_bytes()).await {
        error!("Failed to publish agent message: {}", e);
    } else {
        info!("Published inter-agent message via NATS");
    }

    Ok(())
}

#[cfg(not(feature = "nats"))]
fn wasm_main() -> Result<()> {
    // Initialize simple logger for WASM/Lunatic environment
    simple_logger::SimpleLogger::new().init().unwrap();
    
    log::info!("Starting Rust WASM Lunatic NATS application with supervisor model (WASM-only mode)");
    
    // Create agent configurations for WASM environment
    let agent_configs = vec![
        AgentConfig {
            id: AgentId("worker_agent_1".to_string()),
            memory_backend_type: MemoryBackendType::InMemory,
            nats_enabled: true, // Can enable NATS via WebSocket in WASM mode
            llm_enabled: false,
            agent_type: AgentType::Generic,
        },
        AgentConfig {
            id: AgentId("worker_agent_2".to_string()),
            memory_backend_type: MemoryBackendType::InMemory,
            nats_enabled: true,
            llm_enabled: false,
            agent_type: AgentType::Generic,
        },
        AgentConfig {
            id: AgentId("monitor_agent".to_string()),
            memory_backend_type: MemoryBackendType::InMemory,
            nats_enabled: false,
            llm_enabled: false,
            agent_type: AgentType::Generic,
        },
    ];

    log::info!("Created {} agent configurations for WASM environment", agent_configs.len());

    // Demonstrate the API that would be used in Lunatic
    wasm_demonstrate_supervisor_api(agent_configs)?;

    // Demonstrate WebSocket NATS functionality if available
    #[cfg(feature = "wasm-nats")]
    wasm_demonstrate_websocket_nats()?;

    log::info!("WASM application completed successfully");
    Ok(())
}

#[cfg(not(feature = "nats"))]
fn wasm_demonstrate_supervisor_api(configs: Vec<AgentConfig>) -> Result<()> {
    log::info!("=== Demonstrating Supervisor API (WASM Mode) ===");
    
    for config in &configs {
        log::info!("Agent config: {} (NATS: {})", 
                  config.id.0, config.nats_enabled);
    }

    // Show how the supervisor would be used in Lunatic
    log::info!("In Lunatic WASM environment:");
    log::info!("  let supervisor = spawn_agent_supervisor(configs)?;");
    log::info!("  let agent = spawn_single_agent(config)?;");
    
    // Demonstrate message structures
    let test_message = Message {
        id: "demo_msg_1".to_string(),
        from: AgentId("main".to_string()),
        to: AgentId("worker_agent_1".to_string()),
        payload: serde_json::json!({
            "type": "task",
            "data": "process_data",
            "priority": "high"
        }),
        timestamp: chrono::Utc::now().timestamp() as u64,
    };

    log::info!("Example message: {:?}", test_message);

    // Demonstrate state actions
    let state_action = StateAction::Store {
        key: "config".to_string(),
        value: serde_json::json!({
            "max_workers": 10,
            "timeout": 30,
            "retry_count": 3
        }),
    };

    log::info!("Example state action: {:?}", state_action);

    Ok(())
}

#[cfg(all(not(feature = "nats"), feature = "wasm-nats"))]
fn wasm_demonstrate_websocket_nats() -> Result<()> {
    log::info!("=== Demonstrating WebSocket NATS Integration ===");
    
    // Create WebSocket NATS configuration
    let wasm_nats_config = WasmNatsConfig {
        websocket_url: "ws://localhost:8080/nats".to_string(),
        timeout: std::time::Duration::from_secs(10),
        max_reconnects: Some(5),
        reconnect_delay: std::time::Duration::from_secs(2),
    };
    
    log::info!("WebSocket NATS configuration: {:?}", wasm_nats_config);
    
    // In a real WASM environment, you would:
    log::info!("In WebAssembly environment with WebSocket NATS:");
    log::info!("  let wasm_nats = WasmNatsConnection::new(config).await?;");
    log::info!("  wasm_nats.publish(\"agent.messages\", message_data).await?;");
    log::info!("  let mut receiver = wasm_nats.subscribe(\"system.events\").await?;");
    
    // Demonstrate message structures for WebSocket NATS
    let test_message = serde_json::json!({
        "type": "agent_communication",
        "from": "worker_agent_1",
        "to": "worker_agent_2",
        "payload": {
            "task": "process_data",
            "priority": "high",
            "data": "sample_data"
        },
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    
    log::info!("Example WebSocket NATS message: {}", test_message);
    
    // Show the benefits of WebSocket NATS in WASM
    log::info!("WebSocket NATS Benefits in WASM:");
    log::info!("  ✓ Real-time bidirectional communication");
    log::info!("  ✓ Maintains NATS protocol semantics");
    log::info!("  ✓ Works in browser and server environments");
    log::info!("  ✓ Supports publish/subscribe patterns");
    log::info!("  ✓ Compatible with existing NATS infrastructure");
    
    Ok(())
}

#[cfg(all(not(feature = "nats"), not(feature = "wasm-nats")))]
fn wasm_demonstrate_websocket_nats() -> Result<()> {
    log::info!("=== WebSocket NATS Not Available ===");
    log::info!("To enable WebSocket NATS support, compile with --features wasm-nats");
    log::info!("This would provide NATS messaging capabilities in WebAssembly environments");
    Ok(())
}

// Example of how agents would be used in tests
async fn run_integration_tests() -> Result<()> {
    info!("=== Running Integration Tests ===");

    // This shows how the supervisor pattern would work in tests
    let test_config = AgentConfig {
        id: AgentId("test_agent".to_string()),
        memory_backend_type: MemoryBackendType::InMemory,
        nats_enabled: false,
        llm_enabled: false,
        agent_type: AgentType::Generic,
    };

    info!("Test agent config: {:?}", test_config);

    // In a real Lunatic environment:
    // let agent = spawn_single_agent(test_config)?;
    // send_message_to_agent(&agent, test_message);
    // let state = get_agent_state(&agent);
    // shutdown_agent(&agent);

    info!("Integration tests would run here in WASM environment");
    Ok(())
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_supervisor_config_creation() {
        let config = AgentConfig {
            id: AgentId("test_agent".to_string()),
            memory_backend_type: MemoryBackendType::InMemory,
            nats_enabled: true,
            llm_enabled: false,
            agent_type: AgentType::Generic,
        };
        
        assert_eq!(config.id.0, "test_agent");
        assert!(config.nats_enabled);
    }

    #[tokio::test]
    async fn test_message_serialization() {
        let message = Message {
            id: "test_msg".to_string(),
            from: AgentId("sender".to_string()),
            to: AgentId("receiver".to_string()),
            payload: serde_json::json!({"type": "test", "data": "hello"}),
            timestamp: 12345,
        };

        let serialized = serde_json::to_string(&message).unwrap();
        let deserialized: Message = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(deserialized.id, "test_msg");
        assert_eq!(deserialized.from.0, "sender");
        assert_eq!(deserialized.to.0, "receiver");
    }

    #[tokio::test]
    async fn test_state_action_serialization() {
        let action = StateAction::Store {
            key: "test_key".to_string(),
            value: serde_json::json!({"data": "test_value"}),
        };

        let serialized = serde_json::to_string(&action).unwrap();
        let deserialized: StateAction = serde_json::from_str(&serialized).unwrap();
        
        match deserialized {
            StateAction::Store { key, value } => {
                assert_eq!(key, "test_key");
                assert_eq!(value["data"], "test_value");
            }
            _ => panic!("Expected Store action"),
        }
    }

    #[test]
    fn test_nats_config_from_env() {
        std::env::set_var("NATS_URL", "nats://test:4222");
        
        let config = NatsConfig {
            url: std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string()),
            timeout: Duration::from_secs(10),
            max_reconnects: Some(10),
            reconnect_delay: Duration::from_secs(1),
        };
        
        assert_eq!(config.url, "nats://test:4222");
        
        std::env::remove_var("NATS_URL");
    }
}