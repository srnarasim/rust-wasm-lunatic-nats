//! A sample agent demonstrating basic functionality

use lunatic::Mailbox;
use serde_json::json;

#[lunatic::main]
fn main(_: Mailbox<()>) {
    // Initialize logging
    #[cfg(feature = "logging")]
    simple_logger::SimpleLogger::new().init().unwrap();
    
    log::info!("Starting simple Lunatic agent demonstration");
    
    // Demonstrate basic agent-like functionality
    demonstrate_agent_functionality();
    
    log::info!("Sample agent demonstration completed successfully");
}

fn demonstrate_agent_functionality() {
    log::info!("=== Agent Functionality Demo ===");
    
    // Simple state management with HashMap (simulating the agent state)
    let mut state = std::collections::HashMap::new();
    
    // 1. Initialize state
    state.insert("counter".to_string(), json!(0));
    state.insert("agent_id".to_string(), json!("sample_agent_1"));
    state.insert("status".to_string(), json!("active"));
    log::info!("✅ Initialized agent state with {} keys", state.len());
    
    // 2. Simulate message processing - increment counter
    log::info!("📨 Processing message: increment counter");
    if let Some(current) = state.get("counter").and_then(|v| v.as_i64()) {
        let new_value = current + 1;
        state.insert("counter".to_string(), json!(new_value));
        log::info!("✅ Counter incremented from {} to {}", current, new_value);
    }
    
    // 3. Process another message - set status
    log::info!("📨 Processing message: update status");
    state.insert("status".to_string(), json!("processing"));
    state.insert("last_update".to_string(), json!(chrono::Utc::now().timestamp()));
    log::info!("✅ Status updated to 'processing'");
    
    // 4. List all state keys
    let keys: Vec<&String> = state.keys().collect();
    log::info!("📋 Current state keys: {:?}", keys);
    
    // 5. Show complete state
    log::info!("🔍 Complete agent state:");
    for (key, value) in &state {
        log::info!("   {}: {}", key, value);
    }
    
    // 6. Simulate persistence save
    log::info!("💾 Simulating state persistence...");
    let state_json = serde_json::to_string_pretty(&state).unwrap_or("{}".to_string());
    log::info!("📁 State would be saved as: {}", state_json);
    
    log::info!("✨ Agent functionality demonstration complete");
}
