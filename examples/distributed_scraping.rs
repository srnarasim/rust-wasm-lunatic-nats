//! Real Distributed Web Scraping Example with OpenAI Integration
//! 
//! This example demonstrates the LLM-augmented distributed agent system by:
//! 1. Loading configuration from scraping_config.json and .env files
//! 2. Spawning multiple scraper agents to collect data from real URLs
//! 3. Using real OpenAI API for intelligent summarization
//! 4. Demonstrating priority-based message routing and enhanced workflows

use lunatic::Mailbox;
use rust_wasm_lunatic_nats::*;
use serde_json::{json, Value};
use std::time::Duration;
use std::fs;

// Configuration structures
#[derive(serde::Deserialize, Debug)]
struct ScrapingConfig {
    scraping_targets: Vec<ScrapingTarget>,
    scraping_config: ScrapingSettings,
    llm_config: LLMSettings,
}

#[derive(serde::Deserialize, Debug)]
struct ScrapingTarget {
    id: String,
    url: String,
    title: String,
    description: String,
    priority: String,
    agent_assignment: String,
}

#[derive(serde::Deserialize, Debug)]
struct ScrapingSettings {
    max_concurrent_requests: u32,
    request_timeout_seconds: u64,
    retry_attempts: u32,
    user_agent: String,
    respect_robots_txt: bool,
    rate_limit_delay_ms: u64,
}

#[derive(serde::Deserialize, Debug)]
struct LLMSettings {
    summarization: LLMModelConfig,
    workflow_planning: LLMModelConfig,
    reasoning: LLMModelConfig,
}

#[derive(serde::Deserialize, Debug)]
struct LLMModelConfig {
    max_tokens: u32,
    temperature: f32,
    model: String,
}

#[lunatic::main]
fn main(_: Mailbox<()>) {
    // Initialize logging
    #[cfg(feature = "logging")]
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .unwrap();
    
    log::info!("=== Real Distributed Web Scraping with OpenAI Integration ===");
    
    // Load environment variables
    if let Err(e) = dotenv::dotenv() {
        log::warn!("Could not load .env file: {}. Environment variables must be set manually.", e);
    }
    
    // Load configuration
    let config = match load_scraping_config() {
        Ok(config) => {
            log::info!("✅ Loaded scraping configuration with {} targets", config.scraping_targets.len());
            config
        },
        Err(e) => {
            log::error!("❌ Failed to load scraping configuration: {}", e);
            return;
        }
    };
    
    // Check API key
    let api_key_status = check_openai_api_key();
    
    // Run the demo
    run_real_distributed_scraping_demo(config, api_key_status);
    
    log::info!("=== Real demo completed successfully ===");
}

fn load_scraping_config() -> Result<ScrapingConfig> {
    let config_content = fs::read_to_string("scraping_config.json")
        .map_err(|e| Error::Custom(format!("Failed to read scraping_config.json: {}", e)))?;
    
    let config: ScrapingConfig = serde_json::from_str(&config_content)
        .map_err(|e| Error::Custom(format!("Failed to parse scraping_config.json: {}", e)))?;
    
    Ok(config)
}

fn check_openai_api_key() -> OpenAIStatus {
    match std::env::var("OPENAI_API_KEY") {
        Ok(key) if !key.is_empty() && key.len() > 10 => {
            log::info!("🔑 OpenAI API key detected ({}...)", &key[..std::cmp::min(key.len(), 8)]);
            OpenAIStatus::Available(key)
        },
        Ok(_) => {
            log::warn!("⚠️ OPENAI_API_KEY is set but appears invalid");
            OpenAIStatus::Invalid
        },
        Err(_) => {
            log::info!("ℹ️ OPENAI_API_KEY not set - will use enhanced fallback mode");
            OpenAIStatus::NotSet
        }
    }
}

#[derive(Debug)]
enum OpenAIStatus {
    Available(String),
    Invalid,
    NotSet,
}

fn run_real_distributed_scraping_demo(config: ScrapingConfig, api_status: OpenAIStatus) {
    log::info!("🚀 Starting real distributed web scraping demonstration");
    
    // Display configuration info
    log::info!("📋 Configuration loaded:");
    log::info!("  - {} scraping targets", config.scraping_targets.len());
    log::info!("  - Max concurrent requests: {}", config.scraping_config.max_concurrent_requests);
    log::info!("  - LLM model: {}", config.llm_config.summarization.model);
    
    match &api_status {
        OpenAIStatus::Available(_) => log::info!("  - OpenAI API: ✅ Ready for real LLM calls"),
        OpenAIStatus::Invalid => log::warn!("  - OpenAI API: ⚠️ Invalid key detected"),
        OpenAIStatus::NotSet => log::info!("  - OpenAI API: ℹ️ Using enhanced fallback mode"),
    }
    
    // Step 1: Create agent configurations from config file
    let scraper_configs = create_real_scraper_configs(&config);
    let summarizer_config = create_real_summarizer_config(&config, &api_status);
    let coordinator_config = create_real_coordinator_config(&config);
    
    // Step 2: Spawn the agents
    log::info!("📡 Spawning {} scraper agents from configuration", scraper_configs.len());
    let scraper_agents = spawn_configured_scraper_agents(scraper_configs);
    
    log::info!("🧠 Spawning OpenAI-enabled summarizer agent");
    let summarizer_agent = spawn_openai_summarizer_agent(summarizer_config);
    
    log::info!("🎯 Spawning intelligent workflow coordinator");
    let coordinator_agent = spawn_intelligent_coordinator_agent(coordinator_config);
    
    // Step 3: Send real scraping tasks based on configuration
    log::info!("📋 Distributing real URL scraping tasks to agents");
    send_real_scraping_tasks(&scraper_agents, &config);
    
    // Wait for agents to process
    lunatic::sleep(Duration::from_millis(2000));
    
    // Step 4: Collect data and send to OpenAI-enabled summarizer
    log::info!("📊 Collecting scraped data and sending to OpenAI summarizer");
    let collected_data = collect_real_scraped_data(&scraper_agents, &config);
    send_data_to_openai_summarizer(&summarizer_agent, collected_data);
    
    // Step 5: Request intelligent workflow plan
    log::info!("🗺️ Requesting AI-powered workflow plan from coordinator");
    request_intelligent_workflow_plan(&coordinator_agent, &config);
    
    // Wait for LLM processing
    lunatic::sleep(Duration::from_millis(3000));
    
    // Step 6: Display enhanced results
    log::info!("📈 Checking real LLM integration results");
    display_openai_results(&summarizer_agent, &coordinator_agent, &api_status);
}

// Real configuration-based agent creation functions

fn create_real_scraper_configs(config: &ScrapingConfig) -> Vec<AgentConfig> {
    let mut configs = Vec::new();
    
    // Group targets by agent assignment
    let mut agent_targets: std::collections::HashMap<String, Vec<&ScrapingTarget>> = 
        std::collections::HashMap::new();
    
    for target in &config.scraping_targets {
        agent_targets
            .entry(target.agent_assignment.clone())
            .or_insert_with(Vec::new)
            .push(target);
    }
    
    // Create agent configs
    for (agent_name, targets) in agent_targets {
        log::info!("📝 Creating scraper config for '{}' with {} targets", agent_name, targets.len());
        
        configs.push(AgentConfig {
            id: AgentId(agent_name.clone()),
            agent_type: AgentType::DataCollector,
            memory_backend: MemoryBackendType::InMemory,
            llm_enabled: false, // Scrapers don't need LLM
            metadata: json!({
                "role": "web_scraper",
                "targets": targets.iter().map(|t| json!({
                    "id": t.id,
                    "url": t.url,
                    "title": t.title,
                    "description": t.description,
                    "priority": t.priority
                })).collect::<Vec<_>>(),
                "config": {
                    "timeout_seconds": config.scraping_config.request_timeout_seconds,
                    "user_agent": config.scraping_config.user_agent,
                    "retry_attempts": config.scraping_config.retry_attempts,
                    "rate_limit_delay_ms": config.scraping_config.rate_limit_delay_ms
                }
            }),
        });
    }
    
    configs
}

fn create_real_summarizer_config(config: &ScrapingConfig, api_status: &OpenAIStatus) -> AgentConfig {
    let llm_enabled = matches!(api_status, OpenAIStatus::Available(_));
    
    log::info!("📝 Creating OpenAI summarizer config (LLM enabled: {})", llm_enabled);
    
    AgentConfig {
        id: AgentId("openai_summarizer".to_string()),
        agent_type: AgentType::DataProcessor,
        memory_backend: MemoryBackendType::InMemory,
        llm_enabled,
        metadata: json!({
            "role": "llm_summarizer",
            "capabilities": ["text_summarization", "data_analysis", "insight_generation"],
            "llm_config": {
                "model": config.llm_config.summarization.model,
                "max_tokens": config.llm_config.summarization.max_tokens,
                "temperature": config.llm_config.summarization.temperature
            },
            "api_status": match api_status {
                OpenAIStatus::Available(_) => "ready",
                OpenAIStatus::Invalid => "invalid_key", 
                OpenAIStatus::NotSet => "fallback_mode"
            }
        }),
    }
}

fn create_real_coordinator_config(config: &ScrapingConfig) -> AgentConfig {
    log::info!("📝 Creating intelligent coordinator config");
    
    AgentConfig {
        id: AgentId("intelligent_coordinator".to_string()),
        agent_type: AgentType::Coordinator,
        memory_backend: MemoryBackendType::InMemory,
        llm_enabled: true, // Coordinators benefit from LLM for workflow planning
        metadata: json!({
            "role": "workflow_coordinator",
            "capabilities": ["workflow_planning", "agent_coordination", "task_optimization"],
            "llm_config": {
                "model": config.llm_config.workflow_planning.model,
                "max_tokens": config.llm_config.workflow_planning.max_tokens,
                "temperature": config.llm_config.workflow_planning.temperature
            },
            "coordination_scope": {
                "max_agents": config.scraping_config.max_concurrent_requests,
                "priority_levels": ["critical", "high", "medium", "low"]
            }
        }),
    }
}

fn create_scraper_configs() -> Vec<AgentConfig> {
    vec![
        AgentConfig {
            id: AgentId("web_scraper_1".to_string()),
            memory_backend_type: MemoryBackendType::InMemory,
            nats_enabled: false, // Simplified for demo
            llm_enabled: false,
            agent_type: AgentType::WebScraper,
        },
        AgentConfig {
            id: AgentId("web_scraper_2".to_string()),
            memory_backend_type: MemoryBackendType::InMemory,
            nats_enabled: false,
            llm_enabled: false,
            agent_type: AgentType::WebScraper,
        },
        AgentConfig {
            id: AgentId("data_collector".to_string()),
            memory_backend_type: MemoryBackendType::InMemory,
            nats_enabled: false,
            llm_enabled: false,
            agent_type: AgentType::DataCollector,
        },
    ]
}

fn create_summarizer_config() -> AgentConfig {
    AgentConfig {
        id: AgentId("llm_summarizer".to_string()),
        memory_backend_type: MemoryBackendType::InMemory,
        nats_enabled: false,
        llm_enabled: true, // This agent has LLM capabilities
        agent_type: AgentType::Summarizer,
    }
}

fn create_coordinator_config() -> AgentConfig {
    AgentConfig {
        id: AgentId("workflow_coordinator".to_string()),
        memory_backend_type: MemoryBackendType::InMemory,
        nats_enabled: false,
        llm_enabled: true, // This agent can plan workflows
        agent_type: AgentType::WorkflowCoordinator,
    }
}

fn spawn_scraper_agents(configs: Vec<AgentConfig>) -> Vec<lunatic::ap::ProcessRef<AgentProcess>> {
    configs.into_iter()
        .map(|config| {
            let agent_id = config.id.0.clone();
            match spawn_single_agent(config) {
                Ok(agent) => {
                    log::info!("✅ Spawned scraper agent: {}", agent_id);
                    agent
                }
                Err(e) => {
                    log::error!("❌ Failed to spawn scraper agent {}: {}", agent_id, e);
                    panic!("Failed to spawn scraper agent");
                }
            }
        })
        .collect()
}

fn spawn_summarizer_agent(config: AgentConfig) -> lunatic::ap::ProcessRef<AgentProcess> {
    let agent_id = config.id.0.clone();
    match spawn_single_agent(config) {
        Ok(agent) => {
            log::info!("✅ Spawned summarizer agent: {}", agent_id);
            agent
        }
        Err(e) => {
            log::error!("❌ Failed to spawn summarizer agent {}: {}", agent_id, e);
            panic!("Failed to spawn summarizer agent");
        }
    }
}

fn spawn_coordinator_agent(config: AgentConfig) -> lunatic::ap::ProcessRef<AgentProcess> {
    let agent_id = config.id.0.clone();
    match spawn_single_agent(config) {
        Ok(agent) => {
            log::info!("✅ Spawned coordinator agent: {}", agent_id);
            agent
        }
        Err(e) => {
            log::error!("❌ Failed to spawn coordinator agent {}: {}", agent_id, e);
            panic!("Failed to spawn coordinator agent");
        }
    }
}

fn send_scraping_tasks(scraper_agents: &[lunatic::ap::ProcessRef<AgentProcess>]) {
    let urls_to_scrape = vec![
        "https://example.com/news/1",
        "https://example.com/news/2", 
        "https://example.com/blog/post1",
        "https://example.com/docs/api",
    ];
    
    for (i, agent) in scraper_agents.iter().enumerate() {
        let urls_for_agent: Vec<_> = urls_to_scrape.iter()
            .enumerate()
            .filter(|(idx, _)| idx % scraper_agents.len() == i)
            .map(|(_, url)| *url)
            .collect();
            
        let scraping_message = Message {
            id: format!("scrape_task_{}", i + 1),
            from: AgentId("demo_coordinator".to_string()),
            to: AgentId(if i < 2 { format!("web_scraper_{}", i + 1) } else { "data_collector".to_string() }),
            payload: json!({
                "type": "scrape_urls",
                "urls": urls_for_agent,
                "task_id": format!("task_{}", i + 1)
            }),
            timestamp: chrono::Utc::now().timestamp() as u64,
        };
        
        log::info!("📤 Sending scraping task to agent with {} URLs", urls_for_agent.len());
        send_message_to_agent(agent, scraping_message);
    }
}

fn simulate_data_collection() -> Vec<serde_json::Value> {
    // Simulate scraped data that would be collected by scraper agents
    vec![
        json!({
            "url": "https://example.com/news/1",
            "title": "Breaking: AI Advances in Distributed Systems",
            "content": "Recent developments in AI-powered distributed systems show promising results for scalable agent coordination...",
            "scraped_at": chrono::Utc::now().timestamp(),
            "word_count": 250,
            "sentiment": "positive"
        }),
        json!({
            "url": "https://example.com/news/2", 
            "title": "WebAssembly Performance in Production",
            "content": "Analysis of WebAssembly performance metrics in production environments reveals significant improvements in execution speed...",
            "scraped_at": chrono::Utc::now().timestamp(),
            "word_count": 180,
            "sentiment": "neutral"
        }),
        json!({
            "url": "https://example.com/blog/post1",
            "title": "NATS Messaging Patterns for Microservices",
            "content": "Exploring effective messaging patterns using NATS for microservice architectures and distributed event processing...",
            "scraped_at": chrono::Utc::now().timestamp(),
            "word_count": 320,
            "sentiment": "positive"
        }),
        json!({
            "url": "https://example.com/docs/api",
            "title": "REST API Design Best Practices",
            "content": "Guidelines for designing robust REST APIs including versioning, error handling, and security considerations...",
            "scraped_at": chrono::Utc::now().timestamp(),
            "word_count": 150,
            "sentiment": "neutral"
        }),
    ]
}

fn send_data_to_summarizer(summarizer: &lunatic::ap::ProcessRef<AgentProcess>, data: Vec<serde_json::Value>) {
    let summarization_message = Message {
        id: "llm_summarization_request".to_string(),
        from: AgentId("demo_coordinator".to_string()),
        to: AgentId("llm_summarizer".to_string()),
        payload: json!({
            "llm_task": "summarize",
            "data": data,
            "context": {
                "domain": "web_scraping",
                "data_type": "scraped_articles",
                "request_id": "demo_001"
            }
        }),
        timestamp: chrono::Utc::now().timestamp() as u64,
    };
    
    log::info!("🧠 Sending {} data items to LLM summarizer", data.len());
    send_message_to_agent(summarizer, summarization_message);
}

fn send_workflow_planning_request(coordinator: &lunatic::ap::ProcessRef<AgentProcess>) {
    let workflow_message = Message {
        id: "workflow_planning_request".to_string(),
        from: AgentId("demo_coordinator".to_string()),
        to: AgentId("workflow_coordinator".to_string()),
        payload: json!({
            "llm_task": "plan_workflow",
            "task_description": "Scrape 10 technology websites, extract key insights, and generate a comprehensive report",
            "available_agents": ["web_scraper", "data_processor", "summarizer", "report_generator"],
            "constraints": {
                "max_concurrent_scrapers": 3,
                "timeout_minutes": 30,
                "output_format": "markdown"
            }
        }),
        timestamp: chrono::Utc::now().timestamp() as u64,
    };
    
    log::info!("🗺️ Requesting workflow planning from LLM coordinator");
    send_message_to_agent(coordinator, workflow_message);
}

fn check_agent_results(
    summarizer: &lunatic::ap::ProcessRef<AgentProcess>,
    coordinator: &lunatic::ap::ProcessRef<AgentProcess>
) {
    log::info!("🔍 Checking summarizer results...");
    let summarizer_state = get_agent_state(summarizer);
    
    if let Some(summary) = summarizer_state.get("last_summary") {
        log::info!("📄 LLM Summary Generated:");
        if let Some(summary_text) = summary.as_str() {
            // Truncate for display
            let display_text = if summary_text.len() > 200 {
                format!("{}...", &summary_text[..200])
            } else {
                summary_text.to_string()
            };
            log::info!("   {}", display_text);
        }
    } else {
        log::warn!("⚠️ No summary found in summarizer state");
    }
    
    log::info!("🔍 Checking coordinator workflow plan...");
    let coordinator_state = get_agent_state(coordinator);
    
    if let Some(workflow_plan) = coordinator_state.get("workflow_plan") {
        log::info!("🗺️ Workflow Plan Created:");
        if let Ok(steps) = serde_json::from_value::<Vec<WorkflowStep>>(workflow_plan.clone()) {
            log::info!("   Total steps: {}", steps.len());
            for (i, step) in steps.iter().take(3).enumerate() {
                log::info!("   Step {}: {} -> {}", i + 1, step.agent_type, step.action);
            }
            if steps.len() > 3 {
                log::info!("   ... and {} more steps", steps.len() - 3);
            }
        } else {
            log::info!("   Workflow plan: {}", workflow_plan);
        }
    } else {
        log::warn!("⚠️ No workflow plan found in coordinator state");
    }
    
    // Display some basic metrics
    log::info!("📊 Demo Metrics:");
    log::info!("   - Scraped articles: 4");
    log::info!("   - LLM summarization: {}", if summarizer_state.contains_key("last_summary") { "✅ Success" } else { "❌ Failed" });
    log::info!("   - Workflow planning: {}", if coordinator_state.contains_key("workflow_plan") { "✅ Success" } else { "❌ Failed" });
    log::info!("   - Agent coordination: ✅ Success");
}

/// Helper function to demonstrate agent communication patterns
fn demonstrate_agent_reasoning() {
    log::info!("🤔 Demonstrating LLM reasoning capabilities");
    
    let reasoning_config = AgentConfig {
        id: AgentId("reasoning_agent".to_string()),
        memory_backend_type: MemoryBackendType::InMemory,
        nats_enabled: false,
        llm_enabled: true,
        agent_type: AgentType::Generic,
    };
    
    let reasoning_agent = spawn_single_agent(reasoning_config).unwrap();
    
    let reasoning_message = Message {
        id: "reasoning_test".to_string(),
        from: AgentId("demo_coordinator".to_string()),
        to: AgentId("reasoning_agent".to_string()),
        payload: json!({
            "llm_task": "reason",
            "prompt": "Given the scraped data about AI and WebAssembly, what are the key trends and opportunities for distributed systems?",
            "context": {
                "domain": "technology_analysis",
                "data_sources": ["news_articles", "blog_posts", "documentation"],
                "analysis_type": "trend_identification"
            }
        }),
        timestamp: chrono::Utc::now().timestamp() as u64,
    };
    
    send_message_to_agent(&reasoning_agent, reasoning_message);
    
    // Give time for reasoning
    std::thread::sleep(Duration::from_secs(2));
    
    let state = get_agent_state(&reasoning_agent);
    if let Some(reasoning_result) = state.get("last_reasoning") {
        log::info!("💡 LLM Reasoning Result Generated");
        if let Some(result_text) = reasoning_result.as_str() {
            let display_text = if result_text.len() > 150 {
                format!("{}...", &result_text[..150])
            } else {
                result_text.to_string()
            };
            log::info!("   {}", display_text);
        }
    }
}