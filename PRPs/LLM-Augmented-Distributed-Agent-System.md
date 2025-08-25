# Product Requirements Prompt (PRP): LLM-Augmented Distributed Agent System

## 1. Executive Summary

### Feature Description
Implement a scalable, resilient, and intelligent distributed multi-agent system that leverages Rust WebAssembly (WASM), Lunatic runtime, NATS messaging, and Large Language Model (LLM) integration for autonomous reasoning, planning, and coordination.

### Primary Objectives
- Build fault-tolerant distributed agents with supervisor tree management
- Enable seamless local and distributed messaging via NATS (TCP/WebSocket)
- Integrate LLM APIs for intelligent decision-making and result synthesis
- Provide pluggable state management with automatic persistence
- Demonstrate horizontal scalability across multiple runtime environments

### Success Criteria
- Agents can spawn, communicate, and survive failures automatically
- LLM integration enables intelligent workflow coordination and summarization
- Distributed agents coordinate via NATS messaging for complex tasks
- Demo: Parallel web scraping with LLM-powered result aggregation

### Architecture Layer
**Core WASM + Orchestration + LLM Integration** - This feature spans multiple layers with new LLM integration components.

## 2. Context and Background

### Current System State
The codebase provides a foundation for distributed agent systems with:

**Existing Components:**
- **Agent System** (`src/agent.rs`): Dual architecture with `Agent` handles and `AgentState` for process execution
- **Supervisor System** (`src/supervisor.rs`): Lunatic-based process management with fault tolerance
- **Dual NATS Communication** (`src/nats_comm.rs`, `src/wasm_nats.rs`): Native TCP and WebSocket support
- **Memory Backends** (`src/memory.rs`): Pluggable storage with ephemeral/persistent layers
- **Multi-target builds**: Support for native, WASM-only, and WASM+NATS configurations

**Architecture Patterns:**
- Feature flag-based conditional compilation (`#[cfg(feature = "nats")]`)
- Async/await with `tokio` for native builds
- Serde JSON for serialization across language boundaries
- Error handling with `thiserror` and custom `Result<T>` types
- Structured logging with configurable backends

### Why This Feature is Needed
The system currently lacks:
1. **LLM Integration**: No mechanism for agents to make intelligent decisions or synthesize results
2. **Complex Workflows**: Limited coordination patterns for multi-step distributed tasks
3. **Demonstration Use Cases**: Need concrete examples showing system capabilities
4. **Production-Ready Error Handling**: Enhanced resilience for LLM API failures and network partitions

### Integration with Existing Architecture
This feature extends the current foundation by:
- Adding LLM client modules that integrate with existing agent communication patterns
- Enhancing message types to support LLM reasoning workflows
- Implementing example workflows that demonstrate distributed coordination
- Building on existing NATS infrastructure for reliable message delivery

## 3. Technical Specifications

### Performance Targets
- **Agent Startup Time**: <500ms for lightweight agents, <2s for LLM-enabled agents
- **Message Throughput**: 1000+ messages/sec/agent for local messaging, 100+ for distributed
- **LLM Response Integration**: <5s total time for LLM reasoning cycles
- **Memory Usage**: <10MB per agent (excluding model caches)
- **Fault Recovery**: <1s automatic restart for crashed agents

### Browser Compatibility
- **WASM Target**: `wasm32-wasip1` with Lunatic runtime support
- **WebSocket NATS**: Full protocol compatibility for browser environments
- **Memory Constraints**: Efficient operation within browser memory limits

### Security Considerations
- **API Key Management**: Secure storage and rotation for LLM provider credentials
- **Input Sanitization**: Validation of all LLM prompts and responses
- **Network Security**: TLS encryption for all external API calls
- **Process Isolation**: Leverage WASM sandboxing for agent security boundaries
- **Rate Limiting**: Prevent abuse of LLM APIs and NATS infrastructure

## 4. Implementation Plan

### Step 1: Environment Setup and Dependencies

#### Update Cargo.toml Dependencies
```toml
[dependencies]
# Existing dependencies remain...

# New LLM integration dependencies
reqwest = { version = "0.11", features = ["json", "stream"], optional = true }
openai-api-rs = { version = "5.0", optional = true }
anthropic-sdk = { version = "0.2", optional = true }
tiktoken-rs = { version = "0.5", optional = true }

# Enhanced error handling
uuid = { version = "1.0", features = ["v4", "serde"] }

[features]
default = ["nats", "logging"]
# Existing features...
llm-openai = ["dep:reqwest", "dep:openai-api-rs", "dep:tiktoken-rs"]
llm-anthropic = ["dep:reqwest", "dep:anthropic-sdk"]
llm-all = ["llm-openai", "llm-anthropic"]
```

#### Environment Variables Setup
```bash
# LLM API Configuration
export OPENAI_API_KEY="your-openai-key"
export ANTHROPIC_API_KEY="your-anthropic-key"
export LLM_PROVIDER="openai" # or "anthropic"
export LLM_MODEL="gpt-4" # or "claude-3-sonnet"
export LLM_MAX_TOKENS=1000
export LLM_TIMEOUT_SECONDS=30

# NATS Configuration
export NATS_URL="nats://localhost:4222"
export NATS_WEBSOCKET_URL="ws://localhost:8080"
```

### Step 2: Core LLM Integration Implementation

#### Create `src/llm_client.rs`
```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::{Result, Error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMRequest {
    pub prompt: String,
    pub context: HashMap<String, serde_json::Value>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMResponse {
    pub content: String,
    pub usage: LLMUsage,
    pub provider: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[async_trait::async_trait]
pub trait LLMProvider: Send + Sync {
    async fn complete(&self, request: LLMRequest) -> Result<LLMResponse>;
    fn provider_name(&self) -> &'static str;
}

pub struct LLMClient {
    provider: Box<dyn LLMProvider>,
    default_config: LLMConfig,
}

#[derive(Debug, Clone)]
pub struct LLMConfig {
    pub max_tokens: u32,
    pub temperature: f32,
    pub timeout_seconds: u64,
}

impl Default for LLMConfig {
    fn default() -> Self {
        Self {
            max_tokens: 1000,
            temperature: 0.7,
            timeout_seconds: 30,
        }
    }
}

impl LLMClient {
    pub fn new(provider: Box<dyn LLMProvider>, config: LLMConfig) -> Self {
        Self {
            provider,
            default_config: config,
        }
    }

    pub async fn reasoning_request(&self, prompt: &str, context: HashMap<String, serde_json::Value>) -> Result<String> {
        let request = LLMRequest {
            prompt: prompt.to_string(),
            context,
            max_tokens: Some(self.default_config.max_tokens),
            temperature: Some(self.default_config.temperature),
        };

        let response = self.provider.complete(request).await?;
        Ok(response.content)
    }

    pub async fn summarize_data(&self, data: Vec<serde_json::Value>) -> Result<String> {
        let context = HashMap::from([
            ("task".to_string(), serde_json::json!("summarization")),
            ("data_count".to_string(), serde_json::json!(data.len())),
        ]);

        let prompt = format!(
            "Please analyze and summarize the following {} data items:\n\n{}\n\nProvide a comprehensive summary highlighting key insights and patterns.",
            data.len(),
            serde_json::to_string_pretty(&data)?
        );

        self.reasoning_request(&prompt, context).await
    }

    pub async fn plan_workflow(&self, task_description: &str, available_agents: Vec<String>) -> Result<Vec<WorkflowStep>> {
        let context = HashMap::from([
            ("task".to_string(), serde_json::json!("workflow_planning")),
            ("available_agents".to_string(), serde_json::json!(available_agents)),
        ]);

        let prompt = format!(
            "Given the task: '{}' and available agents: {:?}, create a detailed workflow plan. 
            Respond with a JSON array of workflow steps, each containing: 
            {{\"step_id\": \"string\", \"agent_type\": \"string\", \"action\": \"string\", \"inputs\": [\"string\"], \"outputs\": [\"string\"]}}",
            task_description, available_agents
        );

        let response = self.reasoning_request(&prompt, context).await?;
        let workflow_steps: Vec<WorkflowStep> = serde_json::from_str(&response)
            .map_err(|e| Error::Custom(format!("Failed to parse workflow plan: {}", e)))?;
        
        Ok(workflow_steps)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub step_id: String,
    pub agent_type: String,
    pub action: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
}

// OpenAI Provider Implementation
#[cfg(feature = "llm-openai")]
pub struct OpenAIProvider {
    client: reqwest::Client,
    api_key: String,
    model: String,
}

#[cfg(feature = "llm-openai")]
impl OpenAIProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            model,
        }
    }
}

#[cfg(feature = "llm-openai")]
#[async_trait::async_trait]
impl LLMProvider for OpenAIProvider {
    async fn complete(&self, request: LLMRequest) -> Result<LLMResponse> {
        let openai_request = serde_json::json!({
            "model": self.model,
            "messages": [{
                "role": "user",
                "content": request.prompt
            }],
            "max_tokens": request.max_tokens.unwrap_or(1000),
            "temperature": request.temperature.unwrap_or(0.7)
        });

        let response = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&openai_request)
            .send()
            .await
            .map_err(|e| Error::Custom(format!("OpenAI API request failed: {}", e)))?;

        let response_data: serde_json::Value = response.json().await
            .map_err(|e| Error::Custom(format!("Failed to parse OpenAI response: {}", e)))?;

        let content = response_data["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| Error::Custom("No content in OpenAI response".to_string()))?
            .to_string();

        let usage = response_data["usage"].clone();

        Ok(LLMResponse {
            content,
            usage: serde_json::from_value(usage)
                .unwrap_or(LLMUsage { prompt_tokens: 0, completion_tokens: 0, total_tokens: 0 }),
            provider: "openai".to_string(),
            model: self.model.clone(),
        })
    }

    fn provider_name(&self) -> &'static str {
        "openai"
    }
}

// Factory function for creating LLM clients
pub fn create_llm_client() -> Result<LLMClient> {
    let config = LLMConfig::default();

    #[cfg(feature = "llm-openai")]
    {
        if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
            let model = std::env::var("LLM_MODEL").unwrap_or_else(|_| "gpt-4".to_string());
            let provider = Box::new(OpenAIProvider::new(api_key, model));
            return Ok(LLMClient::new(provider, config));
        }
    }

    Err(Error::Custom("No LLM provider configured".to_string()))
}
```

#### Update `src/agent.rs` for LLM Integration
```rust
// Add to existing imports
use crate::llm_client::{LLMClient, WorkflowStep};

// Add to AgentState struct
impl AgentState {
    pub fn with_llm(mut self, llm_client: LLMClient) -> Self {
        // Store LLM client in persistent state for access across message handling
        let llm_key = "llm_client_config".to_string();
        let llm_config = serde_json::json!({
            "provider": llm_client.provider.provider_name(),
            "enabled": true
        });
        self.ephemeral_state.insert(llm_key, llm_config);
        self.llm_client = Some(llm_client);
        self
    }

    /// LLM-enhanced message processing
    pub async fn handle_llm_message(&mut self, message: Message) -> Result<()> {
        log::debug!("Processing LLM message: {}", message.id);

        if let Some(ref llm_client) = self.llm_client {
            match message.payload.get("llm_task").and_then(|v| v.as_str()) {
                Some("summarize") => {
                    if let Some(data) = message.payload.get("data") {
                        let data_array = data.as_array().unwrap_or(&vec![data.clone()]).clone();
                        let summary = llm_client.summarize_data(data_array).await?;
                        
                        // Store summary in state
                        self.ephemeral_state.insert("last_summary".to_string(), serde_json::json!(summary));
                        
                        // Publish summary via NATS if configured
                        if let Some(ref nats) = self.nats {
                            let summary_msg = Message {
                                id: uuid::Uuid::new_v4().to_string(),
                                from: self.id.clone(),
                                to: AgentId("summary_results".to_string()),
                                payload: serde_json::json!({
                                    "type": "summary_result",
                                    "summary": summary,
                                    "original_data_count": data_array.len()
                                }),
                                timestamp: chrono::Utc::now().timestamp() as u64,
                            };
                            
                            let subject = "results.summaries";
                            let data = serde_json::to_vec(&summary_msg)?;
                            nats.publish(subject, &data).await.map_err(|e| 
                                Error::Custom(format!("Failed to publish summary: {}", e)))?;
                        }

                        log::info!("Agent {} completed summarization task", self.id.0);
                    }
                }
                Some("plan_workflow") => {
                    if let Some(task_desc) = message.payload.get("task_description").and_then(|v| v.as_str()) {
                        let agents = message.payload.get("available_agents")
                            .and_then(|v| v.as_array())
                            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                            .unwrap_or_else(Vec::new);

                        let workflow = llm_client.plan_workflow(task_desc, agents).await?;
                        
                        // Store workflow plan
                        self.ephemeral_state.insert("workflow_plan".to_string(), serde_json::to_value(&workflow)?);
                        
                        log::info!("Agent {} created workflow plan with {} steps", self.id.0, workflow.len());
                    }
                }
                Some("reason") => {
                    if let Some(prompt) = message.payload.get("prompt").and_then(|v| v.as_str()) {
                        let context = message.payload.get("context")
                            .and_then(|v| v.as_object())
                            .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                            .unwrap_or_else(HashMap::new);

                        let reasoning_result = llm_client.reasoning_request(prompt, context).await?;
                        
                        // Store reasoning result
                        self.ephemeral_state.insert("last_reasoning".to_string(), serde_json::json!(reasoning_result));
                        
                        log::info!("Agent {} completed reasoning task", self.id.0);
                    }
                }
                _ => {
                    log::debug!("Unknown LLM task type in message: {:?}", message.payload.get("llm_task"));
                }
            }
        } else {
            log::warn!("Agent {} received LLM message but no LLM client configured", self.id.0);
        }

        Ok(())
    }
}

// Add new field to AgentState
pub struct AgentState {
    pub id: AgentId,
    pub ephemeral_state: HashMap<String, serde_json::Value>,
    pub persistent_backend: Box<dyn MemoryBackend>,
    pub nats: Option<NatsConnection>,
    pub llm_client: Option<LLMClient>, // New field
}
```

### Step 3: Integration with Existing Systems

#### Enhanced Message Types in `src/agent.rs`
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    // Existing types
    StateAction(StateAction),
    ApplicationMessage,
    
    // New LLM-related types
    LLMSummarization {
        data: Vec<serde_json::Value>,
        result_topic: Option<String>,
    },
    LLMWorkflowPlanning {
        task_description: String,
        available_agents: Vec<String>,
    },
    LLMReasoning {
        prompt: String,
        context: HashMap<String, serde_json::Value>,
    },
    WorkflowExecution {
        workflow_id: String,
        current_step: usize,
        steps: Vec<WorkflowStep>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedMessage {
    pub id: String,
    pub from: AgentId,
    pub to: AgentId,
    pub message_type: MessageType,
    pub timestamp: u64,
    pub correlation_id: Option<String>, // For tracking workflow execution
}
```

#### Update Supervisor for LLM-Enabled Agents in `src/supervisor.rs`
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub id: AgentId,
    pub memory_backend_type: MemoryBackendType,
    pub nats_enabled: bool,
    pub llm_enabled: bool, // New field
    pub agent_type: AgentType, // New field for specialized agents
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentType {
    DataCollector,
    Summarizer,
    WorkflowCoordinator,
    WebScraper,
    Generic,
}

// Enhanced spawn function with LLM support
pub async fn spawn_llm_enabled_agent(config: AgentConfig) -> Result<ProcessRef<AgentProcess>> {
    let mut agent_state = AgentState::new(
        config.id.clone(),
        create_memory_backend(config.memory_backend_type.clone())?
    );

    // Add NATS connection if enabled
    if config.nats_enabled {
        let nats_config = NatsConfig::from_env()?;
        let nats_conn = NatsConnection::new(nats_config).await?;
        agent_state = agent_state.with_nats(nats_conn);
    }

    // Add LLM client if enabled
    if config.llm_enabled {
        let llm_client = create_llm_client()?;
        agent_state = agent_state.with_llm(llm_client);
    }

    // Load any existing persistent state
    agent_state.load_persistent_state().await?;

    let agent = AgentProcess::link()
        .start(config)
        .map_err(|_| Error::Custom("Failed to start LLM-enabled agent".to_string()))?;
    
    Ok(agent)
}
```

### Step 4: Error Handling and Validation

#### Enhanced Error Types in `src/lib.rs`
```rust
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

// Enhanced result type with retry logic
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
```

#### Retry Logic for LLM Operations
```rust
pub async fn retry_llm_operation<F, T>(
    operation: F,
    max_retries: u32,
) -> Result<T>
where
    F: Fn() -> futures::future::BoxFuture<'static, Result<T>> + Send + 'static,
    T: Send + 'static,
{
    let mut last_error = Error::Custom("No attempts made".to_string());
    
    for attempt in 0..=max_retries {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(error) if attempt < max_retries && error.is_retryable() => {
                let delay_ms = error.retry_delay_ms();
                log::warn!("LLM operation attempt {} failed: {}. Retrying in {}ms", 
                          attempt + 1, error, delay_ms);
                
                tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                last_error = error;
            }
            Err(error) => return Err(error),
        }
    }
    
    Err(last_error)
}
```

### Step 5: Testing and Optimization

#### Integration Tests in `tests/integration_tests.rs`
```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use tokio::time::{timeout, Duration};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[tokio::test]
    async fn test_llm_enabled_agent_workflow() {
        // Setup test environment
        let config = AgentConfig {
            id: AgentId("llm_test_agent".to_string()),
            memory_backend_type: MemoryBackendType::InMemory,
            nats_enabled: false,
            llm_enabled: true,
            agent_type: AgentType::Summarizer,
        };

        let agent = spawn_llm_enabled_agent(config).await.expect("Failed to spawn agent");

        // Test data for summarization
        let test_data = vec![
            serde_json::json!({"url": "http://example.com/1", "title": "Test Article 1", "content": "Content 1"}),
            serde_json::json!({"url": "http://example.com/2", "title": "Test Article 2", "content": "Content 2"}),
        ];

        // Send LLM summarization message
        let llm_message = Message {
            id: "llm_test_1".to_string(),
            from: AgentId("test_harness".to_string()),
            to: AgentId("llm_test_agent".to_string()),
            payload: serde_json::json!({
                "llm_task": "summarize",
                "data": test_data
            }),
            timestamp: chrono::Utc::now().timestamp() as u64,
        };

        send_message_to_agent(&agent, llm_message);

        // Wait for processing
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Verify summary was created
        let state = get_agent_state(&agent);
        assert!(state.contains_key("last_summary"));
        
        let summary = state.get("last_summary").unwrap();
        assert!(summary.as_str().unwrap().len() > 0);
    }

    #[tokio::test] 
    async fn test_distributed_agent_coordination() {
        // Start NATS server for testing (requires test infrastructure)
        let nats_url = std::env::var("TEST_NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());

        // Create collector agent
        let collector_config = AgentConfig {
            id: AgentId("data_collector".to_string()),
            memory_backend_type: MemoryBackendType::InMemory,
            nats_enabled: true,
            llm_enabled: false,
            agent_type: AgentType::DataCollector,
        };

        // Create summarizer agent
        let summarizer_config = AgentConfig {
            id: AgentId("summarizer".to_string()),
            memory_backend_type: MemoryBackendType::InMemory,
            nats_enabled: true,
            llm_enabled: true,
            agent_type: AgentType::Summarizer,
        };

        let collector = spawn_llm_enabled_agent(collector_config).await.expect("Failed to spawn collector");
        let summarizer = spawn_llm_enabled_agent(summarizer_config).await.expect("Failed to spawn summarizer");

        // Simulate data collection and forwarding
        let collected_data = vec![
            serde_json::json!({"source": "web", "data": "sample data 1"}),
            serde_json::json!({"source": "api", "data": "sample data 2"}),
        ];

        // Collector forwards data to summarizer via NATS
        let forward_message = Message {
            id: "forward_test".to_string(),
            from: AgentId("data_collector".to_string()),
            to: AgentId("summarizer".to_string()),
            payload: serde_json::json!({
                "llm_task": "summarize",
                "data": collected_data
            }),
            timestamp: chrono::Utc::now().timestamp() as u64,
        };

        send_message_to_agent(&collector, forward_message);

        // Wait for distributed processing
        tokio::time::sleep(Duration::from_secs(3)).await;

        // Verify summarizer received and processed the data
        let summarizer_state = get_agent_state(&summarizer);
        assert!(summarizer_state.contains_key("last_summary"));
    }

    #[tokio::test]
    async fn test_workflow_planning_and_execution() {
        let coordinator_config = AgentConfig {
            id: AgentId("workflow_coordinator".to_string()),
            memory_backend_type: MemoryBackendType::InMemory,
            nats_enabled: false,
            llm_enabled: true,
            agent_type: AgentType::WorkflowCoordinator,
        };

        let coordinator = spawn_llm_enabled_agent(coordinator_config).await.expect("Failed to spawn coordinator");

        // Request workflow planning
        let planning_message = Message {
            id: "plan_test".to_string(),
            from: AgentId("test_harness".to_string()),
            to: AgentId("workflow_coordinator".to_string()),
            payload: serde_json::json!({
                "llm_task": "plan_workflow",
                "task_description": "Scrape 10 websites and summarize their content",
                "available_agents": ["web_scraper", "summarizer", "data_validator"]
            }),
            timestamp: chrono::Utc::now().timestamp() as u64,
        };

        send_message_to_agent(&coordinator, planning_message);

        // Wait for planning
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Verify workflow plan was created
        let coordinator_state = get_agent_state(&coordinator);
        assert!(coordinator_state.contains_key("workflow_plan"));

        let workflow_plan = coordinator_state.get("workflow_plan").unwrap();
        let steps: Vec<WorkflowStep> = serde_json::from_value(workflow_plan.clone()).expect("Invalid workflow format");
        assert!(steps.len() > 0);
    }

    #[tokio::test]
    async fn test_fault_tolerance_with_llm_failure() {
        let config = AgentConfig {
            id: AgentId("fault_test_agent".to_string()),
            memory_backend_type: MemoryBackendType::InMemory,
            nats_enabled: false,
            llm_enabled: true,
            agent_type: AgentType::Generic,
        };

        let agent = spawn_llm_enabled_agent(config).await.expect("Failed to spawn agent");

        // Send message that will cause LLM timeout (invalid API key or network issue)
        let timeout_message = Message {
            id: "timeout_test".to_string(),
            from: AgentId("test_harness".to_string()),
            to: AgentId("fault_test_agent".to_string()),
            payload: serde_json::json!({
                "llm_task": "reason",
                "prompt": "This will timeout or fail",
                "context": {}
            }),
            timestamp: chrono::Utc::now().timestamp() as u64,
        };

        send_message_to_agent(&agent, timeout_message);

        // Wait for failure handling
        tokio::time::sleep(Duration::from_secs(1)).await;

        // Verify agent is still responsive after LLM failure
        let ping_message = Message {
            id: "ping_after_failure".to_string(),
            from: AgentId("test_harness".to_string()),
            to: AgentId("fault_test_agent".to_string()),
            payload: serde_json::json!({"type": "ping"}),
            timestamp: chrono::Utc::now().timestamp() as u64,
        };

        send_message_to_agent(&agent, ping_message);

        // Verify agent processed the ping (agent is still alive)
        tokio::time::sleep(Duration::from_millis(100)).await;
        let final_state = get_agent_state(&agent);
        assert!(final_state.contains_key("last_message_from_test_harness"));
    }
}
```

## 5. Code Examples and Patterns

### WebAssembly-JavaScript Interop Pattern for LLM Calls
```rust
// For WASM builds, provide browser-compatible HTTP client
#[cfg(target_arch = "wasm32")]
mod wasm_llm {
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{Request, RequestInit, Response, Headers};

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = console)]
        fn log(s: &str);
    }

    pub async fn fetch_llm_response(url: &str, body: &str, headers: Vec<(&str, &str)>) -> Result<String, JsValue> {
        let mut opts = RequestInit::new();
        opts.method("POST");
        opts.body(Some(&JsValue::from_str(body)));

        let request = Request::new_with_str_and_init(url, &opts)?;

        let header_obj = Headers::new()?;
        for (key, value) in headers {
            header_obj.set(key, value)?;
        }
        request.headers().set("Content-Type", "application/json")?;

        let window = web_sys::window().unwrap();
        let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
        let resp: Response = resp_value.dyn_into().unwrap();

        let json = JsFuture::from(resp.json()?).await?;
        let text = js_sys::JSON::stringify(&json).unwrap();
        
        Ok(text.into())
    }
}

#[cfg(target_arch = "wasm32")]
#[async_trait::async_trait(?Send)]
impl LLMProvider for OpenAIProvider {
    async fn complete(&self, request: LLMRequest) -> Result<LLMResponse> {
        let body = serde_json::json!({
            "model": self.model,
            "messages": [{"role": "user", "content": request.prompt}],
            "max_tokens": request.max_tokens.unwrap_or(1000)
        });

        let headers = vec![
            ("Authorization", &format!("Bearer {}", self.api_key)),
            ("Content-Type", "application/json"),
        ];

        let response_text = wasm_llm::fetch_llm_response(
            "https://api.openai.com/v1/chat/completions",
            &body.to_string(),
            headers
        ).await.map_err(|e| Error::Custom(format!("WASM LLM request failed: {:?}", e)))?;

        // Parse response similar to native implementation
        let response_data: serde_json::Value = serde_json::from_str(&response_text)?;
        
        // Extract content and usage information
        let content = response_data["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| Error::Custom("No content in response".to_string()))?
            .to_string();

        Ok(LLMResponse {
            content,
            usage: serde_json::from_value(response_data["usage"].clone()).unwrap_or_default(),
            provider: "openai".to_string(),
            model: self.model.clone(),
        })
    }

    fn provider_name(&self) -> &'static str {
        "openai"
    }
}
```

### NATS Integration Pattern for Distributed Workflows
```rust
// Enhanced NATS subject routing for workflow coordination
pub struct WorkflowCoordinator {
    agent_state: AgentState,
    active_workflows: HashMap<String, WorkflowExecution>,
}

#[derive(Debug)]
pub struct WorkflowExecution {
    pub id: String,
    pub steps: Vec<WorkflowStep>,
    pub current_step: usize,
    pub results: HashMap<String, serde_json::Value>,
    pub started_at: chrono::DateTime<chrono::Utc>,
}

impl WorkflowCoordinator {
    pub async fn execute_workflow(&mut self, workflow_id: String, steps: Vec<WorkflowStep>) -> Result<()> {
        let execution = WorkflowExecution {
            id: workflow_id.clone(),
            steps: steps.clone(),
            current_step: 0,
            results: HashMap::new(),
            started_at: chrono::Utc::now(),
        };

        self.active_workflows.insert(workflow_id.clone(), execution);

        // Start first step
        self.execute_next_step(&workflow_id).await?;

        Ok(())
    }

    async fn execute_next_step(&mut self, workflow_id: &str) -> Result<()> {
        if let Some(execution) = self.active_workflows.get(workflow_id) {
            if execution.current_step >= execution.steps.len() {
                log::info!("Workflow {} completed successfully", workflow_id);
                self.active_workflows.remove(workflow_id);
                return Ok(());
            }

            let current_step = &execution.steps[execution.current_step];
            log::info!("Executing step {} of workflow {}: {}", 
                      execution.current_step + 1, workflow_id, current_step.action);

            // Route step to appropriate agent type via NATS
            if let Some(ref nats) = self.agent_state.nats {
                let step_message = Message {
                    id: uuid::Uuid::new_v4().to_string(),
                    from: self.agent_state.id.clone(),
                    to: AgentId(format!("{}_{}", current_step.agent_type, workflow_id)),
                    payload: serde_json::json!({
                        "workflow_id": workflow_id,
                        "step_id": current_step.step_id,
                        "action": current_step.action,
                        "inputs": current_step.inputs
                    }),
                    timestamp: chrono::Utc::now().timestamp() as u64,
                };

                let subject = format!("workflow.{}.{}", workflow_id, current_step.agent_type);
                let data = serde_json::to_vec(&step_message)?;
                nats.publish(&subject, &data).await.map_err(|e| {
                    Error::Custom(format!("Failed to publish workflow step: {}", e))
                })?;
            }
        }

        Ok(())
    }

    pub async fn handle_step_completion(&mut self, workflow_id: &str, step_id: &str, result: serde_json::Value) -> Result<()> {
        if let Some(execution) = self.active_workflows.get_mut(workflow_id) {
            execution.results.insert(step_id.to_string(), result);
            execution.current_step += 1;

            // Execute next step
            self.execute_next_step(workflow_id).await?;
        }

        Ok(())
    }
}
```

### Error Handling Across Language Boundaries
```rust
// Robust error handling with detailed context
pub async fn safe_llm_operation<F, T>(
    operation_name: &str,
    agent_id: &AgentId,
    operation: F
) -> Result<T>
where
    F: Fn() -> futures::future::BoxFuture<'static, Result<T>>,
{
    let start_time = std::time::Instant::now();
    
    match retry_llm_operation(operation, 3).await {
        Ok(result) => {
            let duration = start_time.elapsed();
            log::info!("Agent {} completed {} in {:?}", agent_id.0, operation_name, duration);
            Ok(result)
        }
        Err(error) => {
            let duration = start_time.elapsed();
            log::error!("Agent {} failed {} after {:?}: {}", agent_id.0, operation_name, duration, error);
            
            // Create fallback response or graceful degradation
            match operation_name {
                "summarize" => {
                    // Return basic summary without LLM
                    log::warn!("Falling back to simple summarization for agent {}", agent_id.0);
                    // Would implement basic text truncation/extraction here
                }
                "plan_workflow" => {
                    // Return default workflow steps
                    log::warn!("Using default workflow for agent {}", agent_id.0);
                    // Would implement rule-based workflow generation
                }
                _ => {}
            }
            
            Err(error)
        }
    }
}
```

### Memory Management Strategy
```rust
// Enhanced memory backend with LLM response caching
#[async_trait::async_trait]
impl MemoryBackend for LLMCacheBackend {
    async fn store(&mut self, key: &str, value: &serde_json::Value) -> Result<()> {
        // Check if this is an LLM response - implement caching strategy
        if key.starts_with("llm_response:") {
            // Store with TTL for cache invalidation
            let cache_entry = serde_json::json!({
                "value": value,
                "cached_at": chrono::Utc::now().timestamp(),
                "ttl_seconds": 3600 // 1 hour cache
            });
            
            self.inner_backend.store(key, &cache_entry).await?;
        } else {
            self.inner_backend.store(key, value).await?;
        }
        
        Ok(())
    }

    async fn retrieve(&mut self, key: &str) -> Result<Option<serde_json::Value>> {
        if key.starts_with("llm_response:") {
            if let Some(cached_entry) = self.inner_backend.retrieve(key).await? {
                let cached_at = cached_entry["cached_at"].as_i64().unwrap_or(0);
                let ttl = cached_entry["ttl_seconds"].as_i64().unwrap_or(0);
                let now = chrono::Utc::now().timestamp();
                
                if (now - cached_at) < ttl {
                    // Cache hit
                    return Ok(Some(cached_entry["value"].clone()));
                } else {
                    // Cache expired - remove entry
                    self.inner_backend.delete(key).await?;
                    return Ok(None);
                }
            }
        }
        
        self.inner_backend.retrieve(key).await
    }
}
```

## 6. Testing Strategy

### Unit Tests for All Public APIs
```bash
# Run tests for different configurations
cargo test --no-default-features --features wasm-only
cargo test --no-default-features --features "wasm-only,wasm-nats"
cargo test --features "nats,llm-openai"
cargo test --features "nats,llm-all"
```

### Integration Tests for Cross-Language Interactions
```bash
# Requires NATS server and LLM API keys
export TEST_NATS_URL="nats://localhost:4222"
export OPENAI_API_KEY="test-key"
cargo test --features "nats,llm-all" integration_tests
```

### Performance Benchmarks
```bash
# Benchmark agent creation and message throughput
cargo bench --features "nats,llm-all"
```

### Browser Compatibility Tests
```bash
# Build for WASM and test in browser environment
cargo build --target wasm32-wasip1 --no-default-features --features "wasm-only,wasm-nats,llm-all"
wasm-pack test --headless --firefox
```

### Error Scenario Testing
```bash
# Test with invalid API keys, network failures, etc.
OPENAI_API_KEY="invalid" cargo test test_fault_tolerance --features "llm-all"
```

## 7. Success Criteria

### Functional Requirements Met
✅ **Agent Spawning & Management**: Agents start, process messages, and restart after failures  
✅ **NATS Messaging**: Local and distributed communication via TCP/WebSocket  
✅ **LLM Integration**: Reasoning, summarization, and workflow planning capabilities  
✅ **State Persistence**: Automatic sync between ephemeral and persistent storage  
✅ **Fault Tolerance**: Supervisor trees handle crashes and restart agents gracefully  

### Performance Targets Achieved
🎯 **Agent Startup**: <500ms for lightweight, <2s for LLM-enabled  
🎯 **Message Throughput**: 1000+ local messages/sec, 100+ distributed  
🎯 **LLM Integration**: <5s total time for reasoning cycles  
🎯 **Memory Usage**: <10MB per agent baseline  
🎯 **Recovery Time**: <1s automatic restart for crashed agents  

### All Tests Passing
✅ **Unit Tests**: All public APIs have comprehensive test coverage  
✅ **Integration Tests**: Cross-component interactions work correctly  
✅ **Performance Tests**: Benchmarks meet or exceed target metrics  
✅ **Browser Tests**: WASM builds work in browser environments  
✅ **Fault Tolerance**: System recovers gracefully from various failure modes  

### Code Review Approval
✅ **Architecture Review**: Design follows established patterns and best practices  
✅ **Security Review**: Proper handling of API keys, input validation, secure communications  
✅ **Performance Review**: Efficient resource usage, appropriate caching strategies  
✅ **Documentation Review**: Clear API docs, usage examples, troubleshooting guides  

### Documentation Complete
✅ **API Documentation**: Comprehensive rustdoc with usage examples  
✅ **Setup Guide**: Environment configuration and dependency installation  
✅ **Usage Examples**: Working examples for common use cases  
✅ **Troubleshooting**: Common issues and their solutions  
✅ **Architecture Guide**: How components interact and can be extended  

## 8. Validation Commands

### Build Commands
```bash
# Native development build
cargo build --features "nats,llm-all"

# WASM builds
cargo build --target wasm32-wasip1 --no-default-features --features wasm-only
cargo build --target wasm32-wasip1 --no-default-features --features "wasm-only,wasm-nats,llm-all"

# Release builds with optimizations
cargo build --release --features "nats,llm-all"
cargo build --release --target wasm32-wasip1 --no-default-features --features "wasm-only,wasm-nats,llm-all"
```

### Test Commands
```bash
# Unit tests for all configurations
cargo test --lib --features "nats,llm-all"
cargo test --lib --no-default-features --features wasm-only
cargo test --lib --no-default-features --features "wasm-only,wasm-nats"

# Integration tests (requires infrastructure)
cargo test --test integration_tests --features "nats,llm-all"

# Performance benchmarks
cargo bench --features "nats,llm-all"

# Documentation tests
cargo test --doc --features "nats,llm-all"
```

### Lint and Quality Checks
```bash
# Rust linting and formatting
cargo clippy --all-targets --features "nats,llm-all" -- -D warnings
cargo fmt --all -- --check

# Security audit
cargo audit

# Dependency checks
cargo tree --duplicates
cargo outdated
```

### Demo Commands
```bash
# Start infrastructure (requires Docker)
docker run -d -p 4222:4222 nats:latest

# Run example workflow
RUST_LOG=info cargo run --example distributed_scraping --features "nats,llm-all"

# Run WASM demo in browser
wasm-pack build --target web --out-dir www/pkg --features "wasm-only,wasm-nats"
python3 -m http.server 8080 -d www
```

### Validation Checklist
- [ ] All build configurations compile without warnings
- [ ] Unit tests pass with >95% coverage
- [ ] Integration tests pass with external dependencies
- [ ] Performance benchmarks meet target metrics
- [ ] Memory usage stays within defined limits
- [ ] WASM builds work in browser environments
- [ ] LLM integration works with multiple providers
- [ ] NATS messaging works in both TCP and WebSocket modes
- [ ] Agent fault tolerance and recovery functions correctly
- [ ] Documentation is complete and accurate

## 9. Production Deployment Considerations

### Environment Setup
- **NATS Cluster**: Multi-node setup with clustering and persistence
- **LLM Provider Setup**: Production API keys with appropriate rate limits
- **Monitoring**: Prometheus metrics and Grafana dashboards
- **Logging**: Structured logging with log aggregation
- **Security**: TLS everywhere, secure secret management

### Scaling Guidelines
- **Vertical**: Start with 4 CPU, 8GB RAM per node
- **Horizontal**: Add nodes based on message throughput requirements  
- **Geographic**: Deploy edge nodes for reduced latency
- **Resource Planning**: Monitor CPU/memory per agent and plan capacity accordingly

This PRP provides comprehensive guidance for implementing an LLM-augmented distributed agent system that builds upon the existing Rust/WASM/Lunatic/NATS foundation while adding intelligent coordination capabilities.