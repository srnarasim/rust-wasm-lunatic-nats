use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::{Result, Error};
#[cfg(any(feature = "llm-openai", feature = "llm-anthropic"))]
use crate::http_client::{HttpClient, create_http_client, post_json};

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

impl Default for LLMUsage {
    fn default() -> Self {
        Self {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[async_trait::async_trait]
pub trait LLMProvider: Send + Sync {
    async fn complete(&self, request: LLMRequest) -> Result<LLMResponse>;
    fn provider_name(&self) -> &'static str;
}

#[cfg(target_arch = "wasm32")]
#[async_trait::async_trait(?Send)]
pub trait LLMProvider {
    async fn complete(&self, request: LLMRequest) -> Result<LLMResponse>;
    fn provider_name(&self) -> &'static str;
}

pub struct LLMClient {
    provider: Box<dyn LLMProvider>,
    default_config: LLMConfig,
}

impl std::fmt::Debug for LLMClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LLMClient")
            .field("provider", &self.provider.provider_name())
            .field("default_config", &self.default_config)
            .finish()
    }
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

    pub fn provider_name(&self) -> &'static str {
        self.provider.provider_name()
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
    http_client: Box<dyn HttpClient>,
    api_key: String,
    model: String,
}

#[cfg(feature = "llm-openai")]
impl OpenAIProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            http_client: create_http_client(),
            api_key,
            model,
        }
    }
}

#[cfg(all(feature = "llm-openai", not(target_arch = "wasm32")))]
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

        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), format!("Bearer {}", self.api_key));

        let response_data = post_json(
            self.http_client.as_ref(),
            "https://api.openai.com/v1/chat/completions",
            &openai_request,
            headers,
        ).await?;

        let content = response_data["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| Error::Custom("No content in OpenAI response".to_string()))?
            .to_string();

        let usage = response_data["usage"].clone();

        Ok(LLMResponse {
            content,
            usage: serde_json::from_value(usage)
                .unwrap_or_default(),
            provider: "openai".to_string(),
            model: self.model.clone(),
        })
    }

    fn provider_name(&self) -> &'static str {
        "openai"
    }
}

#[cfg(all(feature = "llm-openai", target_arch = "wasm32"))]
#[async_trait::async_trait(?Send)]
impl LLMProvider for OpenAIProvider {
    async fn complete(&self, request: LLMRequest) -> Result<LLMResponse> {
        let openai_request = serde_json::json!({
            "model": self.model,
            "messages": [{
                "role": "user",
                "content": format!("{}\n\nContext: {:?}", request.prompt, request.context)
            }],
            "max_tokens": request.max_tokens.unwrap_or(1000),
            "temperature": request.temperature.unwrap_or(0.7)
        });

        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), format!("Bearer {}", self.api_key));

        let openai_response = post_json(
            self.http_client.as_ref(),
            "https://api.openai.com/v1/chat/completions",
            &openai_request,
            headers,
        ).await?;

        let content = openai_response["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| Error::Custom("Invalid OpenAI response format".to_string()))?;

        Ok(LLMResponse {
            content: content.to_string(),
            usage: LLMUsage::default(),
            provider: "openai".to_string(),
            model: self.model.clone(),
        })
    }

    fn provider_name(&self) -> &'static str {
        "openai"
    }
}

// Mock provider for testing and when no LLM features are enabled
pub struct MockLLMProvider {
    pub responses: HashMap<String, String>,
}

impl MockLLMProvider {
    pub fn new() -> Self {
        let mut responses = HashMap::new();
        responses.insert("summarize".to_string(), "Mock summary: Data analyzed successfully.".to_string());
        responses.insert("plan_workflow".to_string(), r#"[{"step_id": "1", "agent_type": "mock", "action": "process", "inputs": ["data"], "outputs": ["result"]}]"#.to_string());
        responses.insert("reason".to_string(), "Mock reasoning: Task completed with mock logic.".to_string());
        
        Self { responses }
    }

    pub fn with_response(mut self, key: &str, response: &str) -> Self {
        self.responses.insert(key.to_string(), response.to_string());
        self
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[async_trait::async_trait]
impl LLMProvider for MockLLMProvider {
    async fn complete(&self, request: LLMRequest) -> Result<LLMResponse> {
        // Determine response based on prompt content
        let response_key = if request.prompt.contains("summarize") || request.context.get("task").and_then(|v| v.as_str()) == Some("summarization") {
            "summarize"
        } else if request.prompt.contains("workflow") || request.context.get("task").and_then(|v| v.as_str()) == Some("workflow_planning") {
            "plan_workflow"  
        } else {
            "reason"
        };

        let content = self.responses.get(response_key)
            .unwrap_or(&"Mock response: Task processed.".to_string())
            .clone();

        Ok(LLMResponse {
            content,
            usage: LLMUsage {
                prompt_tokens: 10,
                completion_tokens: 20,
                total_tokens: 30,
            },
            provider: "mock".to_string(),
            model: "mock-model".to_string(),
        })
    }

    fn provider_name(&self) -> &'static str {
        "mock"
    }
}

#[cfg(target_arch = "wasm32")]
#[async_trait::async_trait(?Send)]
impl LLMProvider for MockLLMProvider {
    async fn complete(&self, request: LLMRequest) -> Result<LLMResponse> {
        // Determine response based on prompt content
        let response_key = if request.prompt.contains("summarize") || request.context.get("task").and_then(|v| v.as_str()) == Some("summarization") {
            "summarize"
        } else if request.prompt.contains("workflow") || request.context.get("task").and_then(|v| v.as_str()) == Some("workflow_planning") {
            "plan_workflow"  
        } else {
            "reason"
        };

        let content = self.responses.get(response_key)
            .unwrap_or(&"Mock response: Task processed.".to_string())
            .clone();

        Ok(LLMResponse {
            content,
            usage: LLMUsage::default(),
            provider: "mock".to_string(),
            model: "mock-model".to_string(),
        })
    }

    fn provider_name(&self) -> &'static str {
        "mock"
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

    // Fall back to mock provider for development and testing
    log::info!("Using mock LLM provider - configure OPENAI_API_KEY and enable llm-openai feature for real LLM integration");
    let provider = Box::new(MockLLMProvider::new());
    Ok(LLMClient::new(provider, config))
}

// Retry logic for LLM operations
pub async fn retry_llm_operation<F, T, Fut>(
    operation: F,
    max_retries: u32,
) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut last_error = Error::Custom("No attempts made".to_string());
    
    for attempt in 0..=max_retries {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(error) if attempt < max_retries && error.is_retryable() => {
                let delay_ms = error.retry_delay_ms();
                log::warn!("LLM operation attempt {} failed: {}. Retrying in {}ms", 
                          attempt + 1, error, delay_ms);
                
                #[cfg(feature = "nats")]
                tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                
                #[cfg(not(feature = "nats"))]
                {
                    // For WASM builds without tokio, we'll just continue without delay
                    log::debug!("Retrying immediately (no tokio sleep available)");
                }
                
                last_error = error;
            }
            Err(error) => return Err(error),
        }
    }
    
    Err(last_error)
}

// Safe LLM operation wrapper
pub async fn safe_llm_operation<F, T, Fut>(
    operation_name: &str,
    agent_id: &str,
    operation: F
) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let start_time = std::time::Instant::now();
    
    match retry_llm_operation(operation, 3).await {
        Ok(result) => {
            let duration = start_time.elapsed();
            log::info!("Agent {} completed {} in {:?}", agent_id, operation_name, duration);
            Ok(result)
        }
        Err(error) => {
            let duration = start_time.elapsed();
            log::error!("Agent {} failed {} after {:?}: {}", agent_id, operation_name, duration, error);
            Err(error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_llm_provider() {
        let provider = MockLLMProvider::new();
        
        let request = LLMRequest {
            prompt: "Please summarize this data".to_string(),
            context: HashMap::from([("task".to_string(), serde_json::json!("summarization"))]),
            max_tokens: Some(100),
            temperature: Some(0.7),
        };

        let response = provider.complete(request).await.unwrap();
        assert_eq!(response.provider, "mock");
        assert!(response.content.contains("Mock summary"));
    }

    #[tokio::test]
    async fn test_llm_client_summarization() {
        let client = create_llm_client().unwrap();
        
        let test_data = vec![
            serde_json::json!({"title": "Article 1", "content": "Content 1"}),
            serde_json::json!({"title": "Article 2", "content": "Content 2"}),
        ];

        let summary = client.summarize_data(test_data).await.unwrap();
        assert!(!summary.is_empty());
    }

    #[tokio::test]
    async fn test_workflow_planning() {
        let client = create_llm_client().unwrap();
        
        let workflow = client.plan_workflow(
            "Test workflow task",
            vec!["agent1".to_string(), "agent2".to_string()]
        ).await.unwrap();

        assert!(workflow.len() > 0);
        assert!(!workflow[0].step_id.is_empty());
    }

    #[test]
    fn test_workflow_step_serialization() {
        let step = WorkflowStep {
            step_id: "step1".to_string(),
            agent_type: "processor".to_string(),
            action: "process_data".to_string(),
            inputs: vec!["input1".to_string()],
            outputs: vec!["output1".to_string()],
        };

        let serialized = serde_json::to_string(&step).unwrap();
        let deserialized: WorkflowStep = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(step.step_id, deserialized.step_id);
        assert_eq!(step.agent_type, deserialized.agent_type);
    }
}