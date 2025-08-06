# WebSocket NATS Integration for WASM

This guide explains how to use NATS messaging in WebAssembly environments through WebSocket connections.

## Overview

The `rust-wasm-lunatic-nats` project now supports NATS messaging in WASM environments through a WebSocket-based implementation that maintains full NATS protocol compatibility.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    WASM Environment                         │
│                                                             │
│  ┌─────────────────┐    ┌──────────────────────────────┐   │
│  │ Lunatic         │    │ WebSocket NATS Client        │   │
│  │ Supervisor      │◄──►│                              │   │
│  │ Pattern         │    │ - Binary NATS Protocol      │   │
│  │                 │    │ - Pub/Sub Support            │   │
│  │ - Agent Processes   │ - WebSocket Transport        │   │
│  │ - Mailbox System│    │ - Browser Compatible         │   │
│  └─────────────────┘    └──────────────────────────────┘   │
│           │                           │                    │
└───────────┼───────────────────────────┼────────────────────┘
            │                           │
            │          ┌────────────────▼────────────────┐
            │          │     WebSocket Gateway           │
            │          │                                 │
            │          │ - Protocol Translation         │
            │          │ - TLS Termination              │
            │          │ - Connection Management        │
            │          └────────────────┬────────────────┘
            │                           │
            └─────────Local Messaging───┼─────NATS Core────┐
                                        │                  │
                                ┌───────▼──────────────────▼───┐
                                │      NATS Server             │
                                │                              │
                                │ - Traditional TCP Clients   │
                                │ - Clustering                 │
                                │ - JetStream                  │
                                │ - Subject-based Routing      │
                                └──────────────────────────────┘
```

## Feature Configurations

### Available Features

1. **`default = ["nats"]`** - Standard build with full NATS support
2. **`wasm-only = []`** - WASM build without NATS (stub implementations)  
3. **`wasm-nats = [...]`** - WASM build with WebSocket NATS support
4. **`nats = [...]`** - Full native NATS client support

### Build Combinations

```bash
# Regular development build (full NATS)
cargo build

# WASM-only build (no NATS connectivity)
cargo build --target=wasm32-wasip1 --no-default-features --features wasm-only

# WASM build with WebSocket NATS support
cargo build --target=wasm32-wasip1 --no-default-features --features "wasm-only,wasm-nats"

# Tests for different configurations
cargo test --lib --features nats
cargo test --lib --no-default-features --features "wasm-only,wasm-nats"
```

## WebSocket NATS Client Usage

### Basic Configuration

```rust
use rust_wasm_lunatic_nats::wasm_nats::{WasmNatsConfig, WasmNatsConnection, WasmNatsPublisher};

// Configure WebSocket NATS connection
let config = WasmNatsConfig {
    websocket_url: "wss://nats.example.com/ws".to_string(),
    timeout: Duration::from_secs(10),
    max_reconnects: Some(5),
    reconnect_delay: Duration::from_secs(2),
};

// Create connection (requires wasm-nats feature)
#[cfg(feature = "wasm-nats")]
let nats_conn = WasmNatsConnection::new(config).await?;
```

### Publishing Messages

```rust
// Publish raw bytes
let message_data = b"Hello, NATS!";
nats_conn.publish("test.subject", message_data).await?;

// Publish JSON (using trait)
let json_message = serde_json::json!({
    "type": "agent_message",
    "data": "Hello from WASM!",
    "timestamp": chrono::Utc::now().to_rfc3339()
});
nats_conn.publish_json("agent.messages", &json_message).await?;
```

### Subscribing to Messages

```rust
// Subscribe to subjects
let mut receiver = nats_conn.subscribe("system.events").await?;

// Process messages
use futures::StreamExt;
while let Some(message) = receiver.next().await {
    log::info!("Received: {:?}", message);
    
    // Messages are automatically converted to agent::Message format
    match message.payload.get("type") {
        Some(msg_type) => {
            log::info!("Message type: {}", msg_type);
        }
        None => {
            log::warn!("Unknown message format");
        }
    }
}
```

### Connection Management

```rust
// Check connection status
if nats_conn.is_connected() {
    log::info!("WebSocket NATS is connected");
}

// Get connection statistics
let stats = nats_conn.get_stats();
log::info!("Connection stats: {:?}", stats);

// Close connection gracefully
nats_conn.close().await?;
```

## Integration with Lunatic Supervisor Pattern

### Agent Configuration for WASM

```rust
use rust_wasm_lunatic_nats::{AgentConfig, MemoryBackendType, AgentId};

let agent_configs = vec![
    AgentConfig {
        id: AgentId("worker_agent_1".to_string()),
        memory_backend_type: MemoryBackendType::InMemory,
        nats_enabled: true, // Can enable NATS via WebSocket in WASM
    },
    AgentConfig {
        id: AgentId("worker_agent_2".to_string()),
        memory_backend_type: MemoryBackendType::InMemory,
        nats_enabled: true,
    },
];
```

### WASM Main Function Example

```rust
#[cfg(not(feature = "nats"))]
fn main() -> Result<()> {
    log::info!("Starting WASM Lunatic application with WebSocket NATS");
    
    // Demonstrate supervisor pattern
    wasm_demonstrate_supervisor_api(agent_configs)?;
    
    // Demonstrate WebSocket NATS functionality
    #[cfg(feature = "wasm-nats")]
    wasm_demonstrate_websocket_nats()?;
    
    log::info!("WASM application completed successfully");
    Ok(())
}
```

## WebSocket Gateway Setup

To use WebSocket NATS, you need a WebSocket-to-NATS gateway. Here are recommended approaches:

### Option 1: NATS Server Built-in WebSocket Support

NATS Server v2.2+ includes native WebSocket support:

```bash
# nats-server.conf
websocket {
    port: 8080
    no_tls: true
}

# Start NATS server
nats-server -c nats-server.conf
```

Connect to: `ws://localhost:8080`

### Option 2: Third-Party Gateway

Use `nats-websocket-gw`:

```go
package main

import (
    "net/http"
    gw "github.com/nats-io/nats-websocket-gw"
)

func main() {
    gateway := gw.NewGateway(gw.Settings{
        NatsAddr: "localhost:4222",
    })
    
    http.HandleFunc("/nats", gateway.Handler)
    http.ListenAndServe(":8080", nil)
}
```

Connect to: `ws://localhost:8080/nats`

### Option 3: Production Setup (TLS)

For production, use TLS:

```bash
# Configure TLS WebSocket gateway
websocket {
    port: 8443
    tls {
        cert_file: "/path/to/cert.pem"
        key_file: "/path/to/key.pem"
    }
}
```

Connect to: `wss://nats.example.com:8443`

## Protocol Details

### NATS Protocol over WebSocket

The WebSocket NATS client implements the NATS text protocol over WebSocket binary frames:

#### Publishing
```
PUB <subject> <#bytes>\r\n
<payload>\r\n
```

#### Subscribing  
```
SUB <subject> <sid>\r\n
```

#### Message Delivery
```
MSG <subject> <sid> [reply-to] <#bytes>\r\n
<payload>\r\n
```

### Message Format

Messages are automatically converted to the standard agent message format:

```rust
pub struct Message {
    pub id: String,                    // Generated: "nats_{timestamp}"
    pub from: AgentId,                 // Set to: AgentId("nats")
    pub to: AgentId,                   // Set to: AgentId(subject)
    pub payload: serde_json::Value,    // JSON parsed or base64 raw
    pub timestamp: u64,                // Current UTC timestamp
}
```

## Error Handling

### Connection Errors

```rust
match WasmNatsConnection::new(config).await {
    Ok(conn) => {
        log::info!("WebSocket NATS connected successfully");
    }
    Err(e) => {
        log::error!("Failed to connect: {:?}", e);
        // Fallback to local-only messaging
    }
}
```

### Message Processing Errors

```rust
// Messages that can't be parsed as JSON get base64 encoded
if let Some(raw_data) = message.payload.get("raw") {
    let raw_bytes = base64::decode(raw_data.as_str().unwrap_or(""))?;
    // Process raw binary data
}
```

## Performance Considerations

### WebSocket vs TCP NATS

| Aspect | WebSocket NATS | TCP NATS |
|--------|----------------|----------|
| **Latency** | Higher (HTTP overhead) | Lower (direct TCP) |
| **Throughput** | Moderate | High |
| **Browser Support** | ✅ Full | ❌ None |
| **Firewall Friendly** | ✅ HTTP/HTTPS ports | ⚠️ Custom ports |
| **TLS Termination** | Gateway handles | Direct to NATS |
| **Connection Limit** | Browser limit (~6/domain) | System limit |

### Optimization Tips

1. **Use Binary Frames**: WebSocket NATS uses binary frames for efficiency
2. **Connection Pooling**: Reuse connections when possible  
3. **Message Batching**: Combine small messages when feasible
4. **Compression**: Enable WebSocket compression at gateway level
5. **Subject Design**: Use specific subjects to minimize subscription overhead

## Testing

### Unit Tests

```bash
# Test WebSocket NATS configuration
cargo test wasm_nats::tests::test_wasm_nats_config_default

# Test NATS message parsing  
cargo test wasm_nats::tests::test_nats_message_parsing

# Test all WASM features
cargo test --no-default-features --features "wasm-only,wasm-nats"
```

### Integration Testing

```rust
#[cfg(all(test, feature = "wasm-nats"))]
mod integration_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_websocket_nats_publish_subscribe() {
        let config = WasmNatsConfig::default();
        let conn = WasmNatsConnection::new(config).await.unwrap();
        
        // Subscribe first
        let mut receiver = conn.subscribe("test.subject").await.unwrap();
        
        // Publish message
        let test_data = b"integration test";
        conn.publish("test.subject", test_data).await.unwrap();
        
        // Verify message received
        // Note: Requires running WebSocket gateway
    }
}
```

## Troubleshooting

### Common Issues

1. **Connection Failed**
   ```
   Error: Failed to create WebSocket: ...
   ```
   - Verify gateway is running and accessible
   - Check CORS settings for browser environments
   - Confirm URL format (ws:// or wss://)

2. **Message Not Received**
   ```
   Warning: No subscribers for subject
   ```
   - Ensure subscription is established before publishing
   - Verify subject names match exactly
   - Check WebSocket connection status

3. **Build Errors**
   ```
   Error: feature "wasm-nats" not enabled
   ```
   - Add `--features wasm-nats` to build command
   - Verify target is `wasm32-wasip1`

### Debug Configuration

```rust
// Enable debug logging
log::set_max_level(log::LevelFilter::Debug);

// Check connection state
let ready_state = nats_conn.ready_state();
match ready_state {
    0 => log::info!("WebSocket: CONNECTING"),
    1 => log::info!("WebSocket: OPEN"),
    2 => log::info!("WebSocket: CLOSING"), 
    3 => log::info!("WebSocket: CLOSED"),
    _ => log::warn!("WebSocket: UNKNOWN STATE: {}", ready_state),
}
```

## Migration Guide

### From NATS-only to WebSocket NATS

```rust
// Before (NATS-only)
#[cfg(feature = "nats")]
let nats = NatsConnection::new(nats_config).await?;

// After (Conditional WebSocket NATS)
#[cfg(feature = "wasm-nats")]
let wasm_nats = WasmNatsConnection::new(wasm_config).await?;

#[cfg(feature = "nats")]
let nats = NatsConnection::new(nats_config).await?;
```

### Unified Messaging Interface

Consider creating a unified trait:

```rust
#[async_trait]
pub trait MessagingClient {
    async fn publish(&self, subject: &str, data: &[u8]) -> Result<()>;
    async fn subscribe(&self, subject: &str) -> Result<MessageReceiver>;
    fn is_connected(&self) -> bool;
}

// Implement for both NATS types
impl MessagingClient for NatsConnection { ... }
impl MessagingClient for WasmNatsConnection { ... }
```

## Future Enhancements

### Planned Features

1. **JetStream Support**: WebSocket JetStream API integration
2. **Authentication**: JWT and credential-based auth
3. **Reconnection Logic**: Automatic reconnection with exponential backoff
4. **Message Queuing**: Client-side message queuing during disconnections
5. **Compression**: Built-in message compression support

### Contributing

To contribute WebSocket NATS improvements:

1. **Feature Requests**: Open issues with `wasm-nats` label
2. **Protocol Extensions**: Follow NATS protocol specifications  
3. **Testing**: Add both unit and integration tests
4. **Documentation**: Update this guide with new features

## Resources

- [NATS WebSocket Documentation](https://docs.nats.io/running-a-nats-service/configuration/websocket)
- [WebSocket API Reference](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket)
- [Lunatic Runtime Documentation](https://docs.lunatic.solutions/)
- [WASM-Pack Guide](https://rustwasm.github.io/wasm-pack/)