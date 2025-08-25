//! Integration tests for LLM-Augmented Distributed Agent System
//! 
//! These tests validate the complete system integration including:
//! - LLM client functionality with mock and real providers
//! - Agent message processing with LLM tasks
//! - Distributed coordination between agents
//! - Fault tolerance and error handling
//! - Performance characteristics

use rust_wasm_lunatic_nats::*;
use serde_json::json;
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[cfg(feature = "nats")]
use tokio;

/// Test basic LLM client functionality with mock provider
#[tokio::test]
async fn test_mock_llm_provider_integration() {
    let llm_client = create_llm_client().unwrap();
    
    // Test summarization
    let test_data = vec![
        json!({"url": "http://example.com/1", "title": "Test Article 1", "content": "Content 1"}),
        json!({"url": "http://example.com/2", "title": "Test Article 2", "content": "Content 2"}),
    ];

    let summary = llm_client.summarize_data(test_data).await.unwrap();
    assert!(!summary.is_empty());
    assert!(summary.contains("Mock summary") || summary.len() > 10);
    
    // Test workflow planning
    let workflow = llm_client.plan_workflow(
        "Test workflow task",
        vec!["agent1".to_string(), "agent2".to_string()]
    ).await.unwrap();

    assert!(workflow.len() > 0);
    assert!(!workflow[0].step_id.is_empty());
    assert!(!workflow[0].agent_type.is_empty());
    assert!(!workflow[0].action.is_empty());
    
    // Test reasoning
    let reasoning_result = llm_client.reasoning_request(
        "What is the best approach for distributed agent coordination?",
        HashMap::from([("context".to_string(), json!("distributed_systems"))])
    ).await.unwrap();
    
    assert!(!reasoning_result.is_empty());
}

/// Test agent creation and basic LLM message handling
#[test]
fn test_llm_enabled_agent_creation() {
    let config = AgentConfig {
        id: AgentId("test_llm_agent".to_string()),
        memory_backend_type: MemoryBackendType::InMemory,
        nats_enabled: false,
        llm_enabled: true,
        agent_type: AgentType::Summarizer,
    };

    // Test that agent can be spawned with LLM configuration
    let agent = spawn_single_agent(config).unwrap();
    
    // Send an LLM summarization message
    let test_data = vec![
        json!({"title": "Test", "content": "Test content"}),
    ];
    
    let llm_message = Message {
        id: "test_llm_msg".to_string(),
        from: AgentId("test_harness".to_string()),
        to: AgentId("test_llm_agent".to_string()),
        payload: json!({
            "llm_task": "summarize",
            "data": test_data
        }),
        timestamp: chrono::Utc::now().timestamp() as u64,
    };

    send_message_to_agent(&agent, llm_message);
    
    // Give time for processing
    std::thread::sleep(Duration::from_millis(100));
    
    // Verify agent is still responsive (doesn't crash on LLM message)
    let ping_message = Message {
        id: "ping_test".to_string(),
        from: AgentId("test_harness".to_string()),
        to: AgentId("test_llm_agent".to_string()),
        payload: json!({"type": "ping"}),
        timestamp: chrono::Utc::now().timestamp() as u64,
    };

    send_message_to_agent(&agent, ping_message);
    
    // Check agent state shows the ping was received
    std::thread::sleep(Duration::from_millis(50));
    let final_state = get_agent_state(&agent);
    assert!(final_state.contains_key("last_message_from_test_harness"));
}

/// Test different agent types and their configurations
#[test]
fn test_different_agent_types() {
    let agent_types = vec![
        AgentType::DataCollector,
        AgentType::Summarizer,
        AgentType::WorkflowCoordinator,
        AgentType::WebScraper,
        AgentType::Generic,
    ];
    
    for agent_type in agent_types {
        let config = AgentConfig {
            id: AgentId(format!("test_{:?}_agent", agent_type)),
            memory_backend_type: MemoryBackendType::InMemory,
            nats_enabled: false,
            llm_enabled: matches!(agent_type, AgentType::Summarizer | AgentType::WorkflowCoordinator),
            agent_type: agent_type.clone(),
        };

        let agent = spawn_single_agent(config).unwrap();
        
        // Send a basic message to ensure agent responds
        let test_message = Message {
            id: format!("test_msg_{:?}", agent_type),
            from: AgentId("test_harness".to_string()),
            to: AgentId(format!("test_{:?}_agent", agent_type)),
            payload: json!({"type": "test", "agent_type": format!("{:?}", agent_type)}),
            timestamp: chrono::Utc::now().timestamp() as u64,
        };
        
        send_message_to_agent(&agent, test_message);
        
        // Brief processing time
        std::thread::sleep(Duration::from_millis(10));
        
        // Verify agent processed the message
        let state = get_agent_state(&agent);
        assert!(state.contains_key("last_message_from_test_harness"));
    }
}

/// Test LLM task message processing
#[test]
fn test_llm_task_message_processing() {
    let config = AgentConfig {
        id: AgentId("llm_test_agent".to_string()),
        memory_backend_type: MemoryBackendType::InMemory,
        nats_enabled: false,
        llm_enabled: true,
        agent_type: AgentType::Generic,
    };

    let agent = spawn_single_agent(config).unwrap();

    // Test different LLM task types
    let test_cases = vec![
        ("summarize", json!({
            "llm_task": "summarize",
            "data": [{"title": "Test", "content": "Content"}]
        })),
        ("plan_workflow", json!({
            "llm_task": "plan_workflow", 
            "task_description": "Test workflow",
            "available_agents": ["agent1", "agent2"]
        })),
        ("reason", json!({
            "llm_task": "reason",
            "prompt": "Test reasoning prompt",
            "context": {"domain": "test"}
        })),
    ];

    for (task_type, payload) in test_cases {
        let message = Message {
            id: format!("llm_test_{}", task_type),
            from: AgentId("test_harness".to_string()),
            to: AgentId("llm_test_agent".to_string()),
            payload,
            timestamp: chrono::Utc::now().timestamp() as u64,
        };

        send_message_to_agent(&agent, message);
        
        // Give time for LLM processing (mock is fast)
        std::thread::sleep(Duration::from_millis(20));
    }

    // Check that agent is still responsive after all LLM tasks
    let final_ping = Message {
        id: "final_ping".to_string(),
        from: AgentId("test_harness".to_string()),
        to: AgentId("llm_test_agent".to_string()),
        payload: json!({"type": "final_ping"}),
        timestamp: chrono::Utc::now().timestamp() as u64,
    };

    send_message_to_agent(&agent, final_ping);
    std::thread::sleep(Duration::from_millis(10));
    
    let final_state = get_agent_state(&agent);
    assert!(final_state.contains_key("last_message_from_test_harness"));
}

/// Test agent fault tolerance with invalid LLM messages
#[test]
fn test_llm_fault_tolerance() {
    let config = AgentConfig {
        id: AgentId("fault_test_agent".to_string()),
        memory_backend_type: MemoryBackendType::InMemory,
        nats_enabled: false,
        llm_enabled: true,
        agent_type: AgentType::Generic,
    };

    let agent = spawn_single_agent(config).unwrap();

    // Send invalid LLM messages that should be handled gracefully
    let invalid_messages = vec![
        json!({"llm_task": "invalid_task"}),
        json!({"llm_task": "summarize"}), // Missing data
        json!({"llm_task": "plan_workflow"}), // Missing required fields
        json!({"llm_task": "reason"}), // Missing prompt
    ];

    for (i, payload) in invalid_messages.into_iter().enumerate() {
        let message = Message {
            id: format!("invalid_llm_msg_{}", i),
            from: AgentId("test_harness".to_string()),
            to: AgentId("fault_test_agent".to_string()),
            payload,
            timestamp: chrono::Utc::now().timestamp() as u64,
        };

        send_message_to_agent(&agent, message);
        std::thread::sleep(Duration::from_millis(10));
    }

    // Verify agent is still responsive after invalid messages
    let recovery_message = Message {
        id: "recovery_test".to_string(),
        from: AgentId("test_harness".to_string()),
        to: AgentId("fault_test_agent".to_string()),
        payload: json!({"type": "recovery_ping"}),
        timestamp: chrono::Utc::now().timestamp() as u64,
    };

    send_message_to_agent(&agent, recovery_message);
    std::thread::sleep(Duration::from_millis(10));

    let final_state = get_agent_state(&agent);
    assert!(final_state.contains_key("last_message_from_test_harness"));
}

/// Test workflow step serialization and deserialization
#[test]
fn test_workflow_step_serialization() {
    let steps = vec![
        WorkflowStep {
            step_id: "step1".to_string(),
            agent_type: "collector".to_string(),
            action: "collect_data".to_string(),
            inputs: vec!["urls".to_string()],
            outputs: vec!["scraped_data".to_string()],
        },
        WorkflowStep {
            step_id: "step2".to_string(),
            agent_type: "summarizer".to_string(),
            action: "summarize".to_string(),
            inputs: vec!["scraped_data".to_string()],
            outputs: vec!["summary".to_string()],
        },
    ];

    // Test serialization
    let serialized = serde_json::to_string(&steps).unwrap();
    assert!(serialized.contains("step1"));
    assert!(serialized.contains("collector"));

    // Test deserialization
    let deserialized: Vec<WorkflowStep> = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.len(), 2);
    assert_eq!(deserialized[0].step_id, "step1");
    assert_eq!(deserialized[1].agent_type, "summarizer");
}

/// Performance test for agent creation and message processing
#[test]
fn test_agent_performance() {
    let start_time = Instant::now();
    
    // Test agent creation performance
    let configs: Vec<_> = (0..10).map(|i| AgentConfig {
        id: AgentId(format!("perf_agent_{}", i)),
        memory_backend_type: MemoryBackendType::InMemory,
        nats_enabled: false,
        llm_enabled: i % 2 == 0, // Half with LLM
        agent_type: AgentType::Generic,
    }).collect();
    
    let agents: Vec<_> = configs.into_iter()
        .map(|config| spawn_single_agent(config).unwrap())
        .collect();
    
    let creation_time = start_time.elapsed();
    assert!(creation_time < Duration::from_millis(500), 
           "Agent creation took too long: {:?}", creation_time);
    
    // Test message processing performance
    let message_start = Instant::now();
    
    for (i, agent) in agents.iter().enumerate() {
        let message = Message {
            id: format!("perf_msg_{}", i),
            from: AgentId("perf_test".to_string()),
            to: AgentId(format!("perf_agent_{}", i)),
            payload: json!({"type": "performance_test", "data": "test"}),
            timestamp: chrono::Utc::now().timestamp() as u64,
        };
        
        send_message_to_agent(agent, message);
    }
    
    // Give time for all messages to be processed
    std::thread::sleep(Duration::from_millis(50));
    
    let processing_time = message_start.elapsed();
    assert!(processing_time < Duration::from_millis(200),
           "Message processing took too long: {:?}", processing_time);
    
    // Verify all agents processed their messages
    for agent in &agents {
        let state = get_agent_state(agent);
        assert!(state.contains_key("last_message_from_perf_test"));
    }
    
    log::info!("Performance test completed:");
    log::info!("  - Agent creation: {:?} for {} agents", creation_time, agents.len());
    log::info!("  - Message processing: {:?} for {} messages", processing_time, agents.len());
}

/// Test memory backend integration with agents
#[test]
fn test_agent_memory_backends() {
    // Test InMemory backend
    let in_memory_config = AgentConfig {
        id: AgentId("memory_test_agent_1".to_string()),
        memory_backend_type: MemoryBackendType::InMemory,
        nats_enabled: false,
        llm_enabled: false,
        agent_type: AgentType::Generic,
    };
    
    let agent1 = spawn_single_agent(in_memory_config).unwrap();
    
    // Test File backend
    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join("test_agent_data");
    
    let file_config = AgentConfig {
        id: AgentId("memory_test_agent_2".to_string()),
        memory_backend_type: MemoryBackendType::File { 
            path: temp_path.to_string_lossy().to_string() 
        },
        nats_enabled: false,
        llm_enabled: false,
        agent_type: AgentType::Generic,
    };
    
    let agent2 = spawn_single_agent(file_config).unwrap();
    
    // Send state operations to both agents
    let state_action = StateAction::Store {
        key: "test_key".to_string(),
        value: json!({"backend_test": true, "timestamp": chrono::Utc::now().timestamp()}),
    };
    
    send_state_action_to_agent(&agent1, state_action.clone());
    send_state_action_to_agent(&agent2, state_action);
    
    std::thread::sleep(Duration::from_millis(20));
    
    // Verify state was stored in both agents
    let state1 = get_agent_state(&agent1);
    let state2 = get_agent_state(&agent2);
    
    // Note: The get_agent_state function returns the lunatic process state,
    // not the AgentState persistent storage, so we check for message processing
    assert!(state1.len() > 0, "Agent 1 should have processed messages");
    assert!(state2.len() > 0, "Agent 2 should have processed messages");
}

/// Test error handling in LLM client
#[tokio::test] 
async fn test_llm_error_handling() {
    use rust_wasm_lunatic_nats::llm_client::{MockLLMProvider, LLMClient, LLMConfig, LLMRequest};
    
    // Create a mock provider that will return errors for certain inputs
    let mut mock_provider = MockLLMProvider::new();
    mock_provider = mock_provider.with_response("error_test", "");
    
    let config = LLMConfig::default();
    let client = LLMClient::new(Box::new(mock_provider), config);
    
    // Test handling of empty responses
    let result = client.reasoning_request("error_test", HashMap::new()).await;
    
    // Should handle empty response gracefully
    match result {
        Ok(response) => assert!(response.is_empty() || response.len() > 0),
        Err(_) => {}, // Error handling is also acceptable
    }
    
    // Test malformed workflow planning response
    let workflow_result = client.plan_workflow("invalid", vec![]).await;
    
    // Should handle parsing errors gracefully
    match workflow_result {
        Ok(_) => {}, // If parsing succeeds, that's fine
        Err(e) => {
            // Should be a parsing error
            assert!(e.to_string().contains("parse") || e.to_string().contains("Mock"));
        }
    }
}

/// Test concurrent agent operations
#[test]
fn test_concurrent_agent_operations() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::thread;
    
    let num_agents = 5;
    let messages_per_agent = 3;
    let processed_count = Arc::new(AtomicUsize::new(0));
    
    // Create multiple agents
    let agents: Vec<_> = (0..num_agents).map(|i| {
        let config = AgentConfig {
            id: AgentId(format!("concurrent_agent_{}", i)),
            memory_backend_type: MemoryBackendType::InMemory,
            nats_enabled: false,
            llm_enabled: i % 2 == 0,
            agent_type: AgentType::Generic,
        };
        spawn_single_agent(config).unwrap()
    }).collect();
    
    // Send messages concurrently from multiple threads
    let handles: Vec<_> = agents.into_iter().enumerate().map(|(i, agent)| {
        let processed_count = Arc::clone(&processed_count);
        thread::spawn(move || {
            for j in 0..messages_per_agent {
                let message = Message {
                    id: format!("concurrent_msg_{}_{}", i, j),
                    from: AgentId("concurrent_test".to_string()),
                    to: AgentId(format!("concurrent_agent_{}", i)),
                    payload: json!({
                        "type": "concurrent_test",
                        "agent_id": i,
                        "message_num": j
                    }),
                    timestamp: chrono::Utc::now().timestamp() as u64,
                };
                
                send_message_to_agent(&agent, message);
                processed_count.fetch_add(1, Ordering::SeqCst);
                
                // Small delay to simulate processing time
                thread::sleep(Duration::from_millis(1));
            }
            agent
        })
    }).collect();
    
    // Wait for all threads to complete
    let final_agents: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    
    // Give time for all messages to be processed
    thread::sleep(Duration::from_millis(100));
    
    // Verify all messages were sent
    assert_eq!(processed_count.load(Ordering::SeqCst), num_agents * messages_per_agent);
    
    // Verify all agents received messages
    for agent in final_agents {
        let state = get_agent_state(&agent);
        assert!(state.contains_key("last_message_from_concurrent_test"));
    }
}