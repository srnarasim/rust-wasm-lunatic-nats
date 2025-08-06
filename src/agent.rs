use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::{Result, Error};
use crate::memory::MemoryBackend;
use crate::nats_comm::NatsConnection;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub from: AgentId,
    pub to: AgentId,
    pub payload: serde_json::Value,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StateAction {
    Store { key: String, value: serde_json::Value },
    Get { key: String },
    Delete { key: String },
    Clear,
    List,
}

// Lightweight agent handle for external API compatibility
#[derive(Debug)]
pub struct Agent {
    pub id: AgentId,
}

impl Agent {
    pub fn new(id: String) -> Self {
        Self {
            id: AgentId(id),
        }
    }

    pub fn get_id(&self) -> &AgentId {
        &self.id
    }
}

// Agent state for use within processes (compatible with existing code)
#[derive(Debug)]
pub struct AgentState {
    pub id: AgentId,
    pub ephemeral_state: HashMap<String, serde_json::Value>,
    pub persistent_backend: Box<dyn MemoryBackend>,
    pub nats: Option<NatsConnection>,
}

impl AgentState {
    pub fn new(id: AgentId, persistent_backend: Box<dyn MemoryBackend>) -> Self {
        Self {
            id,
            ephemeral_state: HashMap::new(),
            persistent_backend,
            nats: None,
        }
    }

    pub fn with_nats(mut self, nats: NatsConnection) -> Self {
        self.nats = Some(nats);
        self
    }

    /// Load persistent state into ephemeral cache on startup
    pub async fn load_persistent_state(&mut self) -> Result<()> {
        let prefix = format!("{}:", self.id.0);
        let keys = self.persistent_backend.list_keys(Some(&prefix)).await?;

        for key in keys {
            if let Some(local_key) = key.strip_prefix(&prefix) {
                if let Some(value) = self.persistent_backend.retrieve(&key).await? {
                    self.ephemeral_state.insert(local_key.to_string(), value);
                }
            }
        }

        log::info!("Loaded {} state entries for agent {}", 
                  self.ephemeral_state.len(), self.id.0);
        Ok(())
    }

    /// Save ephemeral state to persistent backend
    pub async fn save_persistent_state(&mut self) -> Result<()> {
        for (key, value) in &self.ephemeral_state {
            let persistent_key = format!("{}:{}", self.id.0, key);
            self.persistent_backend.store(&persistent_key, value).await?;
        }

        log::info!("Saved {} state entries for agent {}", 
                  self.ephemeral_state.len(), self.id.0);
        Ok(())
    }

    /// Handle state operations - always operate on ephemeral state first
    pub async fn handle_state_action(&mut self, action: StateAction) -> Result<()> {
        match action {
            StateAction::Store { key, value } => {
                // Store in ephemeral state immediately
                self.ephemeral_state.insert(key.clone(), value.clone());
                
                // Persist to backend
                let persistent_key = format!("{}:{}", self.id.0, key);
                self.persistent_backend.store(&persistent_key, &value).await?;
                
                log::debug!("Stored state: {} = {:?}", key, value);
            }
            StateAction::Get { key } => {
                // First check ephemeral state
                if let Some(value) = self.ephemeral_state.get(&key) {
                    log::debug!("Retrieved from ephemeral state: {} = {:?}", key, value);
                    return Ok(());
                }

                // Then check persistent backend
                let persistent_key = format!("{}:{}", self.id.0, key);
                if let Some(value) = self.persistent_backend.retrieve(&persistent_key).await? {
                    // Cache in ephemeral state
                    self.ephemeral_state.insert(key.clone(), value.clone());
                    log::debug!("Retrieved from persistent state: {} = {:?}", key, value);
                } else {
                    log::debug!("State key not found: {}", key);
                }
            }
            StateAction::Delete { key } => {
                // Remove from ephemeral state
                self.ephemeral_state.remove(&key);
                
                // Remove from persistent backend
                let persistent_key = format!("{}:{}", self.id.0, key);
                self.persistent_backend.delete(&persistent_key).await?;
                
                log::debug!("Deleted state: {}", key);
            }
            StateAction::Clear => {
                // Clear ephemeral state
                self.ephemeral_state.clear();
                
                // Clear persistent state
                self.persistent_backend.clear().await?;
                
                log::debug!("Cleared all state for agent {}", self.id.0);
            }
            StateAction::List => {
                let keys: Vec<String> = self.ephemeral_state.keys().cloned().collect();
                log::info!("Agent {} state keys: {:?}", self.id.0, keys);
            }
        }

        Ok(())
    }

    /// Process incoming messages
    pub async fn handle_message(&mut self, message: Message) -> Result<()> {
        log::debug!("Agent {} processing message: {}", self.id.0, message.id);

        // Check if this is a state action
        if let Ok(state_action) = serde_json::from_value::<StateAction>(message.payload.clone()) {
            return self.handle_state_action(state_action).await;
        }

        // Handle NATS forwarding for inter-node communication
        if let Some(ref nats) = self.nats {
            if message.to.0 != self.id.0 {
                // Forward message via NATS if it's for another agent
                let subject = format!("agent.{}", message.to.0);
                let data = serde_json::to_vec(&message)?;
                nats.publish(&subject, &data).await.map_err(|e| {
                    Error::Custom(format!("NATS publish failed: {}", e))
                })?;
                
                log::debug!("Forwarded message via NATS to {}", message.to.0);
                return Ok(());
            }
        }

        // Process message payload (customize based on your application needs)
        self.process_application_message(&message).await?;

        Ok(())
    }

    /// Application-specific message processing
    async fn process_application_message(&mut self, message: &Message) -> Result<()> {
        // Store the last message in ephemeral state
        let state_key = format!("last_message_from_{}", message.from.0);
        self.ephemeral_state.insert(state_key, message.payload.clone());

        // Example: Handle different message types
        if let Some(msg_type) = message.payload.get("type") {
            match msg_type.as_str() {
                Some("ping") => {
                    log::info!("Agent {} received ping from {}", self.id.0, message.from.0);
                    // Could send pong response here
                }
                Some("data_update") => {
                    if let Some(data) = message.payload.get("data") {
                        self.ephemeral_state.insert("received_data".to_string(), data.clone());
                        log::info!("Agent {} updated data from {}", self.id.0, message.from.0);
                    }
                }
                Some("shutdown") => {
                    log::info!("Agent {} received shutdown signal", self.id.0);
                    self.save_persistent_state().await?;
                    return Err(Error::Custom("Shutdown requested".to_string()));
                }
                _ => {
                    log::debug!("Agent {} received unknown message type: {:?}", self.id.0, msg_type);
                }
            }
        }

        Ok(())
    }

    pub fn get_id(&self) -> &AgentId {
        &self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::InMemoryBackend;

    #[test]
    fn test_agent_id_creation() {
        let agent_id = AgentId("test_agent".to_string());
        assert_eq!(agent_id.0, "test_agent");
    }

    #[test]
    fn test_message_creation() {
        let message = Message {
            id: "test_msg".to_string(),
            from: AgentId("sender".to_string()),
            to: AgentId("receiver".to_string()),
            payload: serde_json::json!({"type": "test"}),
            timestamp: 12345,
        };
        
        assert_eq!(message.id, "test_msg");
        assert_eq!(message.from.0, "sender");
        assert_eq!(message.to.0, "receiver");
    }

    #[test]
    fn test_state_action_serialization() {
        let action = StateAction::Store {
            key: "test_key".to_string(),
            value: serde_json::json!({"data": "test"}),
        };
        
        let serialized = serde_json::to_string(&action).unwrap();
        let deserialized: StateAction = serde_json::from_str(&serialized).unwrap();
        
        match deserialized {
            StateAction::Store { key, value } => {
                assert_eq!(key, "test_key");
                assert_eq!(value, serde_json::json!({"data": "test"}));
            }
            _ => panic!("Expected Store action"),
        }
    }

    #[cfg(feature = "nats")]
    #[tokio::test]
    async fn test_agent_state_operations() {
        let backend = Box::new(InMemoryBackend::new());
        let mut agent_state = AgentState::new(
            AgentId("test_agent".to_string()),
            backend,
        );

        // Test store operation
        let store_action = StateAction::Store {
            key: "test_key".to_string(),
            value: serde_json::json!({"data": "test_value"}),
        };
        
        agent_state.handle_state_action(store_action).await.unwrap();
        
        // Verify it was stored in ephemeral state
        assert!(agent_state.ephemeral_state.contains_key("test_key"));
        assert_eq!(
            agent_state.ephemeral_state.get("test_key").unwrap(),
            &serde_json::json!({"data": "test_value"})
        );
    }

    #[cfg(feature = "nats")]
    #[tokio::test]
    async fn test_agent_state_persistence() {
        let backend = Box::new(InMemoryBackend::new());
        let mut agent_state = AgentState::new(
            AgentId("persist_agent".to_string()),
            backend,
        );

        // Store some state
        agent_state.ephemeral_state.insert(
            "persist_key".to_string(),
            serde_json::json!({"important": "data"}),
        );

        // Save to persistent backend
        agent_state.save_persistent_state().await.unwrap();

        // Clear ephemeral state
        agent_state.ephemeral_state.clear();

        // Load from persistent backend
        agent_state.load_persistent_state().await.unwrap();

        // Verify data was restored
        assert!(agent_state.ephemeral_state.contains_key("persist_key"));
        assert_eq!(
            agent_state.ephemeral_state.get("persist_key").unwrap(),
            &serde_json::json!({"important": "data"})
        );
    }

    #[cfg(feature = "nats")]
    #[tokio::test]
    async fn test_message_processing() {
        let backend = Box::new(InMemoryBackend::new());
        let mut agent_state = AgentState::new(
            AgentId("msg_agent".to_string()),
            backend,
        );

        // Create a state action message
        let message = Message {
            id: "msg_1".to_string(),
            from: AgentId("external".to_string()),
            to: AgentId("msg_agent".to_string()),
            payload: serde_json::to_value(StateAction::Store {
                key: "message_key".to_string(),
                value: serde_json::json!({"from_message": true}),
            }).unwrap(),
            timestamp: 12345,
        };

        // Process the message
        agent_state.handle_message(message).await.unwrap();

        // Verify the state was updated
        assert!(agent_state.ephemeral_state.contains_key("message_key"));
    }
}