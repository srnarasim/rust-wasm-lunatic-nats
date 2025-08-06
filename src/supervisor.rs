use lunatic::ap::handlers::{Message, Request};
use lunatic::ap::{AbstractProcess, Config, MessageHandler, ProcessRef, RequestHandler, State};
use lunatic::supervisor::{Supervisor, SupervisorConfig, SupervisorStrategy};
use lunatic::serializer::Json;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::agent::{AgentId, Message as AgentMessage, StateAction};

// Agent configuration for spawning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub id: AgentId,
    pub memory_backend_type: MemoryBackendType,
    pub nats_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryBackendType {
    InMemory,
    File { path: String },
}

// Agent process that implements AbstractProcess
#[derive(Debug)]
pub struct AgentProcess {
    id: AgentId,
    state: HashMap<String, serde_json::Value>,
    message_count: u32,
}

impl AbstractProcess for AgentProcess {
    type Arg = AgentConfig;
    type State = AgentProcess;
    type Serializer = Json;
    type Handlers = (
        Message<AgentMessage>,
        Message<StateAction>,
        Request<GetAgentState>,
        Message<Shutdown>,
    );
    type StartupError = ();

    fn init(_config: Config<Self>, arg: Self::Arg) -> std::result::Result<Self::State, ()> {
        log::info!("Initializing agent process: {}", arg.id.0);
        
        Ok(AgentProcess {
            id: arg.id,
            state: HashMap::new(),
            message_count: 0,
        })
    }

    fn terminate(state: Self::State) {
        log::info!("Agent {} terminating gracefully", state.id.0);
    }
}

// Message handlers for AgentProcess
impl MessageHandler<AgentMessage> for AgentProcess {
    fn handle(mut state: State<Self>, message: AgentMessage) {
        state.message_count += 1;
        log::info!("Agent {} received message #{}: {}", 
                  state.id.0, state.message_count, message.id);
        
        // Store the last message
        let key = format!("last_message_from_{}", message.from.0);
        state.state.insert(key, message.payload);
    }
}

impl MessageHandler<StateAction> for AgentProcess {
    fn handle(mut state: State<Self>, action: StateAction) {
        match action {
            StateAction::Store { key, value } => {
                state.state.insert(key.clone(), value.clone());
                log::debug!("Agent {} stored state: {} = {:?}", state.id.0, key, value);
            }
            StateAction::Get { key } => {
                if let Some(value) = state.state.get(&key) {
                    log::debug!("Agent {} retrieved state: {} = {:?}", state.id.0, key, value);
                } else {
                    log::debug!("Agent {} state key not found: {}", state.id.0, key);
                }
            }
            StateAction::Delete { key } => {
                state.state.remove(&key);
                log::debug!("Agent {} deleted state: {}", state.id.0, key);
            }
            StateAction::Clear => {
                state.state.clear();
                log::debug!("Agent {} cleared all state", state.id.0);
            }
            StateAction::List => {
                let keys: Vec<String> = state.state.keys().cloned().collect();
                log::info!("Agent {} state keys: {:?}", state.id.0, keys);
            }
        }
    }
}

// Request to get agent state
#[derive(Serialize, Deserialize)]
pub struct GetAgentState;

impl RequestHandler<GetAgentState> for AgentProcess {
    type Response = HashMap<String, serde_json::Value>;

    fn handle(state: State<Self>, _request: GetAgentState) -> Self::Response {
        state.state.clone()
    }
}

// Shutdown message
#[derive(Serialize, Deserialize)]
pub struct Shutdown;

impl MessageHandler<Shutdown> for AgentProcess {
    fn handle(state: State<Self>, _msg: Shutdown) {
        log::info!("Agent {} received shutdown signal", state.id.0);
        // The process will terminate after this handler completes
    }
}

// Supervisor implementation
pub struct AgentSupervisor {
    configs: Vec<AgentConfig>,
}

impl AgentSupervisor {
    pub fn new(configs: Vec<AgentConfig>) -> Self {
        Self { configs }
    }
}

impl Supervisor for AgentSupervisor {
    type Arg = Vec<AgentConfig>;
    type Children = (AgentProcess,); // Can be extended to (AgentProcess, AgentProcess, ...)

    fn init(config: &mut SupervisorConfig<Self>, configs: Self::Arg) {
        log::info!("Initializing supervisor with {} agent configs", configs.len());
        
        config.set_strategy(SupervisorStrategy::OneForOne);
        
        // For simplicity, we'll just use the first config
        // In a real implementation, you would need to handle multiple configs
        if let Some(agent_config) = configs.first() {
            config.set_args((agent_config.clone(),));
        }
    }
}

// Helper functions
pub fn spawn_agent_supervisor(configs: Vec<AgentConfig>) -> std::result::Result<ProcessRef<AgentSupervisor>, crate::Error> {
    let supervisor = AgentSupervisor::link()
        .start(configs)
        .map_err(|_| crate::Error::Custom("Failed to start supervisor".to_string()))?;
    
    Ok(supervisor)
}

pub fn spawn_single_agent(config: AgentConfig) -> std::result::Result<ProcessRef<AgentProcess>, crate::Error> {
    let agent = AgentProcess::link()
        .start(config)
        .map_err(|_| crate::Error::Custom("Failed to start agent".to_string()))?;
    
    Ok(agent)
}

// Convenience functions for agent communication
pub fn send_message_to_agent(agent: &ProcessRef<AgentProcess>, message: AgentMessage) {
    agent.send(message);
}

pub fn send_state_action_to_agent(agent: &ProcessRef<AgentProcess>, action: StateAction) {
    agent.send(action);
}

pub fn get_agent_state(agent: &ProcessRef<AgentProcess>) -> HashMap<String, serde_json::Value> {
    agent.request(GetAgentState)
}

pub fn shutdown_agent(agent: &ProcessRef<AgentProcess>) {
    agent.send(Shutdown);
}

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
    use super::*;
    use lunatic::test;
    use std::time::Duration;

    #[test]
    fn test_agent_spawn_and_message() {
        let config = AgentConfig {
            id: AgentId("test_agent".to_string()),
            memory_backend_type: MemoryBackendType::InMemory,
            nats_enabled: false,
        };

        let agent = spawn_single_agent(config).unwrap();
        
        // Send a test message
        let test_message = AgentMessage {
            id: "test_msg_1".to_string(),
            from: AgentId("test_sender".to_string()),
            to: AgentId("test_agent".to_string()),
            payload: serde_json::json!({"type": "test", "data": "hello"}),
            timestamp: 12345,
        };
        
        send_message_to_agent(&agent, test_message);
        
        // Give some time for message processing
        lunatic::sleep(Duration::from_millis(10));
        
        // Get agent state
        let state = get_agent_state(&agent);
        assert!(state.contains_key("last_message_from_test_sender"));
    }

    #[test]
    fn test_agent_state_operations() {
        let config = AgentConfig {
            id: AgentId("state_test_agent".to_string()),
            memory_backend_type: MemoryBackendType::InMemory,
            nats_enabled: false,
        };

        let agent = spawn_single_agent(config).unwrap();
        
        // Store state
        let store_action = StateAction::Store {
            key: "test_key".to_string(),
            value: serde_json::json!({"data": "test_value"}),
        };
        send_state_action_to_agent(&agent, store_action);
        
        // Give some time for processing
        lunatic::sleep(Duration::from_millis(10));
        
        // Get agent state
        let state = get_agent_state(&agent);
        assert!(state.contains_key("test_key"));
        assert_eq!(state.get("test_key").unwrap(), &serde_json::json!({"data": "test_value"}));
    }

    #[test]
    fn test_supervisor_spawn() {
        let configs = vec![
            AgentConfig {
                id: AgentId("supervised_agent_1".to_string()),
                memory_backend_type: MemoryBackendType::InMemory,
                nats_enabled: false,
            }
        ];

        let _supervisor = spawn_agent_supervisor(configs).unwrap();
        
        // Give supervisor time to start
        lunatic::sleep(Duration::from_millis(10));
        
        // Try to lookup the supervised agent
        if let Some(agent) = ProcessRef::<AgentProcess>::lookup("supervised_agent_1") {
            // Send a message to the supervised agent
            let test_message = AgentMessage {
                id: "supervised_test".to_string(),
                from: AgentId("test".to_string()),
                to: AgentId("supervised_agent_1".to_string()),
                payload: serde_json::json!({"supervised": true}),
                timestamp: 12345,
            };
            send_message_to_agent(&agent, test_message);
        }
    }
}