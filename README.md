# Rust WASM Lunatic NATS - Distributed Agent System

A powerful Rust-based distributed agent system designed for WASM compilation using Lunatic runtime with comprehensive NATS messaging support. This project provides a foundation for building scalable, distributed agent-based applications that work seamlessly across native and WebAssembly environments.

## 🚀 Features

### Core Architecture
- **Agent-Based Architecture**: Create and manage intelligent agents with persistent state
- **Lunatic Supervisor Pattern**: Fault-tolerant agent lifecycle management with automatic restarts
- **Memory Management**: Pluggable memory backends (in-memory and file-based persistence)
- **Process Isolation**: Agents run as separate WASM processes with memory isolation

### Messaging & Communication
- **Dual-Layer Messaging**: Local Lunatic mailboxes for fast communication + distributed NATS messaging
- **Native NATS Support**: Full NATS integration using modern async-nats client (TCP)
- **WebSocket NATS Support**: NATS messaging in WASM environments via WebSocket protocol
- **Real-time Patterns**: Pub/sub, request/reply, and streaming patterns
- **Connection Management**: Auto-reconnection, stats tracking, and graceful degradation

### WASM & Runtime Support
- **Multi-Target Builds**: Native, WASM-only, and WASM+WebSocket NATS configurations
- **Lunatic Runtime**: Actor-based WASM execution with supervisor trees
- **Browser Compatible**: WebSocket NATS works in browser environments
- **Type Safety**: Comprehensive error handling with `thiserror`
- **Async/Await**: Full async support with conditional Tokio integration

## 🏗️ Architecture

``` mermaid
%%{init: {
    "theme": "default",
    "themeVariables": {
        "fontFamily": "Inter, Roboto, sans-serif",
        "fontSize": "14px",
        "primaryColor": "#E0F7FA",
        "primaryBorderColor": "#00ACC1",
        "primaryTextColor": "#004D40",
        "secondaryColor": "#F1F8E9",
        "secondaryBorderColor": "#7CB342",
        "secondaryTextColor": "#33691E",
        "tertiaryColor": "#FFF3E0",
        "tertiaryBorderColor": "#FB8C00",
        "tertiaryTextColor": "#E65100",
        "storageColor": "#EDE7F6",
        "storageBorderColor": "#5E35B1",
        "storageTextColor": "#311B92",
        "nodeBorderRadius": "8px"
    }
}}%%

flowchart TB
    %% --- Build Configurations ---
    subgraph build["Build Configurations"]
        style build fill:#E0F7FA,stroke:#00ACC1,stroke-width:2px,color:#004D40
        native["Native Build<br/><small>(Full NATS)</small>"]
        wasmOnly["WASM-Only Build<br/><small>(No External Connectivity)</small>"]
        wasmNats["WASM + WebSocket NATS Build<br/><small>(WebSocket NATS Support)</small>"]
    end

    build --> runtime

    %% --- Lunatic Runtime ---
    subgraph runtime["Lunatic Runtime"]
        style runtime fill:#F1F8E9,stroke:#7CB342,stroke-width:2px,color:#33691E
        subgraph supervisor["Main Supervisor Process"]
            spawn["Agent Spawning"]
            health["Health Monitor"]
            fault["Fault Recovery System"]
        end

        supervisor --> agentA
        supervisor --> agentB
        supervisor --> agentC
        supervisor --> agentN

        subgraph agentA["Agent A Process<br/><small>(WASM)</small>"]
            stateA["Ephemeral State"]
        end
        subgraph agentB["Agent B Process<br/><small>(WASM)</small>"]
            stateB["Ephemeral State"]
        end
        subgraph agentC["Agent C Process<br/><small>(WASM)</small>"]
            stateC["Ephemeral State"]
        end
        subgraph agentN["Agent N Process<br/><small>(WASM)</small>"]
            stateN["Ephemeral State"]
        end
    end

    %% --- Communication Layer ---
    agentA --> comm
    agentB --> comm
    agentC --> comm
    agentN --> comm

    subgraph comm["Communication Layer"]
        style comm fill:#FFF3E0,stroke:#FB8C00,stroke-width:2px,color:#E65100
        subgraph mailbox["Lunatic Mailboxes<br/><small>(Local / Fast)</small>"]
            mailboxDetails["Process-to-Process<br/>Supervisor Commands<br/>State Sync"]
        end
        subgraph nats["NATS Messaging"]
            subgraph tcpNats["Native TCP NATS Client"]
                tcpDetails["High Performance<br/> Full API"]
            end
            subgraph wsNats["WebSocket NATS Client<br/><small>(WASM)</small>"]
                wsDetails["Browser Compatible"]
            end
        end
    end

    comm --> wsGateway

    %% --- WebSocket Gateway ---
    subgraph wsGateway["WebSocket Gateway<br/><small>(Optional)</small>"]
        style wsGateway fill:#FFF3E0,stroke:#FB8C00,stroke-width:2px,color:#E65100
        proto["Protocol Translation"]
        tls["TLS Termination &<br/>Load Balancing"]
        proto <--> tls
    end

    wsGateway --> natsCluster

    %% --- NATS Server Cluster ---
    subgraph natsCluster["NATS Server Cluster"]
        style natsCluster fill:#FFF3E0,stroke:#FB8C00,stroke-width:2px,color:#E65100
        routing["Subject-Based Routing"]
        jetstream["JetStream & Persistence<br/><small>(Future Enhancement)</small>"]
    end

    natsCluster --> storage

    %% --- Persistent Storage ---
    subgraph storage["Persistent Storage"]
        style storage fill:#EDE7F6,stroke:#5E35B1,stroke-width:2px,color:#311B92
        mem["In-Memory Backend"]
        file["File-Based Backend<br/><small>(Configurable)</small>"]
    end

```

## 📁 Project Structure

```
src/
├── lib.rs          # Library exports and common types
├── main.rs         # Application entry point with multi-config demos
├── agent.rs        # Agent implementation with state management
├── memory.rs       # Memory backend traits and implementations
├── nats_comm.rs    # Native NATS communication layer (TCP)
├── wasm_nats.rs    # WebSocket NATS client for WASM environments
└── supervisor.rs   # Lunatic supervisor implementation

docs/
├── ARCHITECTURE.md    # Detailed architecture documentation
└── WASM_NATS_GUIDE.md # Complete WebSocket NATS integration guide

scripts/            # Build and deployment scripts
```

## 🎯 Build Configurations

### Feature Flags

| Feature | Description | Target Environment |
|---------|-------------|-------------------|
| `default = ["nats"]` | Full native NATS support | Native development |
| `wasm-only = []` | WASM without external connectivity | Lunatic runtime (local) |
| `wasm-nats = [...]` | WebSocket NATS for WASM | WASM + external messaging |
| `nats = [...]` | Native TCP NATS client | Production native |

### Build Commands

```bash
# Native development (full NATS)
cargo build
cargo run

# WASM-only build (no external connectivity)
cargo build --target=wasm32-wasip1 --no-default-features --features wasm-only

# WASM with WebSocket NATS support
cargo build --target=wasm32-wasip1 --no-default-features --features "wasm-only,wasm-nats"

# Test different configurations
cargo test --features nats                                    # Native NATS tests
cargo test --no-default-features --features "wasm-only,wasm-nats"  # WASM NATS tests
```

## 🚀 Quick Start

### Prerequisites

- Rust 1.70+ with 2021 edition
- Cargo package manager
- NATS server (optional - graceful degradation when unavailable)
- WebSocket gateway (for WASM NATS features)

### Installation & Building

```bash
# Clone the repository
git clone <repository-url>
cd rust-wasm-lunatic-nats

# Build for native development
cargo build

# Run comprehensive tests
cargo test

# Run demo application (works with/without NATS)
cargo run
```

### WASM Compilation

```bash
# Add WASM target
rustup target add wasm32-wasip1

# Build WASM with WebSocket NATS support
cargo build --target=wasm32-wasip1 --no-default-features --features "wasm-only,wasm-nats"

# Run with Lunatic runtime (requires lunatic CLI)
lunatic run target/wasm32-wasip1/debug/rust-wasm-lunatic-nats.wasm
```

## 💡 Usage Examples

### 1. Native Lunatic Supervisor with NATS

```rust
use rust_wasm_lunatic_nats::{
    AgentConfig, AgentId, MemoryBackendType,
    spawn_single_agent, send_message_to_agent
};

#[lunatic::main]
fn main(_: lunatic::Mailbox<()>) -> Result<(), Box<dyn std::error::Error>> {
    let agent_config = AgentConfig {
        id: AgentId("my_agent".to_string()),
        memory_backend_type: MemoryBackendType::InMemory,
        nats_enabled: true,
    };
    
    // Spawn agent in Lunatic supervisor
    let agent_process = spawn_single_agent(agent_config)?;
    
    // Send message via Lunatic mailbox
    let message = Message {
        id: "msg_1".to_string(),
        from: AgentId("main".to_string()),
        to: AgentId("my_agent".to_string()),
        payload: serde_json::json!({"task": "process_data"}),
        timestamp: chrono::Utc::now().timestamp() as u64,
    };
    
    send_message_to_agent(&agent_process, message);
    
    Ok(())
}
```

### 2. WebSocket NATS in WASM Environment

```rust
use rust_wasm_lunatic_nats::wasm_nats::{WasmNatsConfig, WasmNatsConnection, WasmNatsPublisher};

#[cfg(feature = "wasm-nats")]
async fn wasm_nats_example() -> Result<(), Box<dyn std::error::Error>> {
    // Configure WebSocket NATS
    let config = WasmNatsConfig {
        websocket_url: "wss://nats.example.com/ws".to_string(),
        timeout: Duration::from_secs(10),
        max_reconnects: Some(5),
        reconnect_delay: Duration::from_secs(2),
    };
    
    // Connect to NATS via WebSocket
    let nats_conn = WasmNatsConnection::new(config).await?;
    
    // Publish JSON message
    let message = serde_json::json!({
        "type": "agent_communication",
        "from": "wasm_agent_1",
        "to": "agent_2",
        "payload": {"task": "data_processing", "priority": "high"}
    });
    
    nats_conn.publish_json("agent.messages", &message).await?;
    
    // Subscribe and process messages
    let mut receiver = nats_conn.subscribe("system.events").await?;
    
    // In real application, this would be an event loop
    if let Some(msg) = receiver.try_next()? {
        log::info!("Received: {:?}", msg);
    }
    
    Ok(())
}
```

### 3. Hybrid Architecture (Local + Distributed)

```rust
// This demonstrates the dual-layer messaging approach
async fn hybrid_messaging_example() -> Result<()> {
    // Local Lunatic communication (fast)
    let local_agent = spawn_single_agent(local_config)?;
    send_message_to_agent(&local_agent, local_message);
    
    // Distributed NATS communication (across nodes)
    #[cfg(feature = "nats")]
    {
        let nats = NatsConnection::new(nats_config).await?;
        nats.publish("global.events", distributed_message.as_bytes()).await?;
    }
    
    // WebSocket NATS for WASM environments
    #[cfg(feature = "wasm-nats")]
    {
        let wasm_nats = WasmNatsConnection::new(wasm_config).await?;
        wasm_nats.publish("wasm.events", wasm_message.as_bytes()).await?;
    }
    
    Ok(())
}
```

### 4. Agent State Management

```rust
use rust_wasm_lunatic_nats::{AgentState, StateAction, InMemoryBackend};

async fn state_management_example() -> Result<()> {
    let backend = Box::new(InMemoryBackend::new());
    let mut agent_state = AgentState::new(
        AgentId("stateful_agent".to_string()),
        backend,
    );
    
    // Store persistent data
    let store_action = StateAction::Store {
        key: "user_preferences".to_string(),
        value: serde_json::json!({
            "theme": "dark",
            "notifications": true,
            "language": "en"
        }),
    };
    
    agent_state.handle_state_action(store_action).await?;
    
    // Retrieve data
    let get_action = StateAction::Get {
        key: "user_preferences".to_string(),
    };
    
    agent_state.handle_state_action(get_action).await?;
    
    Ok(())
}
```

## ⚙️ Configuration

### Environment Variables

```bash
# NATS server configuration
export NATS_URL="nats://localhost:4222"           # Native NATS server
export NATS_WEBSOCKET_URL="ws://localhost:8080"   # WebSocket gateway

# Logging configuration
export RUST_LOG="info"                            # Log level
export RUST_LOG_STYLE="always"                    # Force colored output
```

### NATS Server Setup

#### Option 1: Native NATS Server
```bash
# Install and run NATS server
brew install nats-server  # macOS
nats-server               # Start server on default port 4222
```

#### Option 2: WebSocket Gateway for WASM
```bash
# NATS server with WebSocket support
cat > nats-websocket.conf << EOF
websocket {
    port: 8080
    no_tls: true
}
EOF

nats-server -c nats-websocket.conf
```

#### Option 3: Production Setup (TLS)
```bash
# Production configuration with TLS
cat > nats-production.conf << EOF
websocket {
    port: 8443
    tls {
        cert_file: "/path/to/cert.pem"
        key_file: "/path/to/key.pem"
    }
}

# Standard NATS port
port: 4222
EOF

nats-server -c nats-production.conf
```

### Docker Setup

```yaml
# docker-compose.yml
version: '3.8'
services:
  nats:
    image: nats:2.10-alpine
    ports:
      - "4222:4222"    # Native NATS
      - "8080:8080"    # WebSocket
    command: 
      - "--websocket"
      - "--port=4222"
      - "--websocket_port=8080"
  
  app:
    build: .
    depends_on:
      - nats
    environment:
      - NATS_URL=nats://nats:4222
      - NATS_WEBSOCKET_URL=ws://nats:8080
```

## 🧪 Testing & Development

### Running Tests

```bash
# Test all configurations
cargo test                                                          # Native tests
cargo test --no-default-features --features wasm-only              # WASM-only tests  
cargo test --no-default-features --features "wasm-only,wasm-nats"  # WASM+WebSocket tests

# Test specific modules
cargo test agent::tests        # Agent functionality
cargo test wasm_nats::tests   # WebSocket NATS client
cargo test supervisor::tests  # Lunatic supervisor (WASM target only)

# Integration tests with logging
RUST_LOG=debug cargo test --lib
```

### Code Quality & Documentation

```bash
# Code formatting
cargo fmt

# Linting
cargo clippy

# Generate documentation
cargo doc --open

# Documentation with all features
cargo doc --all-features --open
```

### Performance Testing

```bash
# Build optimized release
cargo build --release

# WASM release build
cargo build --target=wasm32-wasip1 --release --no-default-features --features "wasm-only,wasm-nats"

# Benchmark tests (if available)
cargo bench
```

## 📊 API Reference

### Core Types

```rust
// Agent identification
pub struct AgentId(pub String);

// Message structure for all communication
pub struct Message {
    pub id: String,
    pub from: AgentId, 
    pub to: AgentId,
    pub payload: serde_json::Value,
    pub timestamp: u64,
}

// State management commands
pub enum StateAction {
    Store { key: String, value: serde_json::Value },
    Get { key: String },
    Delete { key: String },
    Clear,
    List,
}
```

### Supervisor API

```rust
// Lunatic supervisor functions
pub fn spawn_agent_supervisor(configs: Vec<AgentConfig>) -> Result<ProcessRef<AgentSupervisor>>;
pub fn spawn_single_agent(config: AgentConfig) -> Result<ProcessRef<AgentProcess>>;

// Agent communication
pub fn send_message_to_agent(agent: &ProcessRef<AgentProcess>, message: Message);
pub fn send_state_action_to_agent(agent: &ProcessRef<AgentProcess>, action: StateAction);
pub fn get_agent_state(agent: &ProcessRef<AgentProcess>) -> HashMap<String, serde_json::Value>;
pub fn shutdown_agent(agent: &ProcessRef<AgentProcess>);
```

### NATS Communication APIs

```rust
// Native NATS (TCP)
impl NatsConnection {
    pub async fn new(config: NatsConfig) -> Result<Self>;
    pub async fn publish(&self, subject: &str, data: &[u8]) -> Result<()>;
    pub async fn subscribe(&self, subject: &str) -> Result<Vec<Message>>;
    pub fn get_stats(&self) -> ConnectionStats;
}

// WebSocket NATS (WASM)
impl WasmNatsConnection {
    pub async fn new(config: WasmNatsConfig) -> Result<Self>;
    pub async fn publish(&self, subject: &str, data: &[u8]) -> Result<()>;
    pub async fn subscribe(&self, subject: &str) -> Result<UnboundedReceiver<Message>>;
    pub fn is_connected(&self) -> bool;
    pub fn get_stats(&self) -> WasmConnectionStats;
}

// JSON convenience trait (available for both)
pub trait NatsPublisher {
    async fn publish_json<T: Serialize>(&self, subject: &str, data: &T) -> Result<()>;
}
```

### Memory Backend Trait

```rust
#[async_trait]
pub trait MemoryBackend: Send + Sync + std::fmt::Debug {
    async fn store(&mut self, key: &str, value: &Value) -> Result<()>;
    async fn retrieve(&mut self, key: &str) -> Result<Option<Value>>;
    async fn delete(&mut self, key: &str) -> Result<bool>;
    async fn list_keys(&self, prefix: Option<&str>) -> Result<Vec<String>>;
    async fn clear(&mut self) -> Result<()>;
}

// Implementations
pub struct InMemoryBackend;  // Fast, ephemeral storage
pub struct FileBackend;      // Persistent file-based storage
```

## 🗺️ Roadmap

### ✅ Completed Features
- **Lunatic Supervisor Pattern**: Complete fault-tolerant agent management
- **Native NATS Integration**: Full TCP NATS client with async-nats
- **WebSocket NATS Support**: WASM-compatible NATS messaging
- **Multi-Target Builds**: Native, WASM-only, and WASM+NATS configurations
- **State Management**: Dual-layer ephemeral + persistent storage
- **Process Isolation**: Separate WASM processes with mailbox communication
- **Comprehensive Testing**: All build configurations tested

### 🚧 In Progress
- [ ] **JetStream Support**: Advanced messaging patterns and persistence
- [ ] **Agent Discovery**: Service discovery and registration via NATS
- [ ] **Load Balancing**: Distribute agents across multiple Lunatic nodes

### 🔮 Planned Features
- [ ] **Monitoring & Metrics**: Prometheus integration and health checks
- [ ] **Security Enhancement**: JWT authentication and authorization
- [ ] **Clustering Support**: Multi-node deployment with leader election
- [ ] **Performance Optimization**: Zero-copy message passing where possible
- [ ] **Browser SDK**: Direct browser agent support via WebSocket NATS

### 📈 Performance Targets
- [ ] **Latency**: <1ms for local Lunatic messages, <10ms for NATS messages
- [ ] **Throughput**: 10K+ messages/sec for local, 1K+ messages/sec for NATS
- [ ] **Scalability**: 1000+ agents per Lunatic runtime instance

## 🤝 Contributing

We welcome contributions! Please see our contribution guidelines:

### Development Setup
```bash
# Fork and clone the repository
git clone https://github.com/yourusername/rust-wasm-lunatic-nats.git
cd rust-wasm-lunatic-nats

# Install development dependencies
rustup component add clippy rustfmt
rustup target add wasm32-wasip1

# Run pre-commit checks
cargo fmt --all
cargo clippy --all-targets --all-features
cargo test --all-features
```

### Contribution Process
1. **Fork the repository**
2. **Create a feature branch** (`git checkout -b feature/amazing-feature`)
3. **Make your changes** with comprehensive tests
4. **Ensure all tests pass** across all build configurations
5. **Update documentation** as needed
6. **Commit your changes** (`git commit -m 'Add amazing feature'`)
7. **Push to the branch** (`git push origin feature/amazing-feature`)
8. **Open a Pull Request** with detailed description

### Areas for Contribution
- **WebSocket Gateway Implementations**: Additional gateway options
- **Memory Backends**: New storage implementations (Redis, Database, etc.)
- **Monitoring Integration**: Metrics and observability enhancements
- **Documentation**: Examples, tutorials, and API documentation
- **Performance**: Benchmarks and optimization improvements

## 📄 License

This project is dual-licensed under the MIT OR Apache-2.0 License. See [LICENSE](LICENSE) file for details.

## 🔗 Dependencies

### Core Runtime
- **lunatic**: Actor-based WASM runtime with supervisor trees
- **tokio**: Async runtime (conditionally compiled)
- **futures**: Stream processing and async utilities

### Messaging
- **async-nats**: Modern NATS client with full async support
- **ws_stream_wasm**: WebSocket streams for WASM environments
- **web-sys**: Browser WebSocket API bindings

### Serialization & Data
- **serde**: Serialization framework with derive macros
- **serde_json**: JSON serialization support
- **bytes**: Efficient byte buffer management
- **base64**: Binary data encoding for WebSocket messages

### Utilities & Error Handling
- **thiserror**: Structured error handling
- **chrono**: Date and time handling with serialization
- **env_logger**: Configurable logging output
- **anyhow**: Error context and chaining

## 📖 Additional Resources

### Documentation
- **[WASM NATS Integration Guide](docs/WASM_NATS_GUIDE.md)**: Complete WebSocket NATS setup and usage
- **[Architecture Documentation](docs/ARCHITECTURE.md)**: Detailed system design and patterns
- **[API Documentation](https://docs.rs/rust-wasm-lunatic-nats)**: Generated API reference

### External Resources
- **[Lunatic Documentation](https://docs.lunatic.solutions/)**: Actor-based WASM runtime
- **[NATS Documentation](https://docs.nats.io/)**: NATS messaging system
- **[WebAssembly Guide](https://webassembly.org/)**: WebAssembly standards and tools

### Examples Repository
Run `cargo run` to see comprehensive examples demonstrating:
- ✅ Multi-configuration builds and feature detection
- ✅ Agent creation and lifecycle management  
- ✅ State persistence and retrieval across restarts
- ✅ Native NATS and WebSocket NATS messaging
- ✅ Connection statistics and health monitoring
- ✅ Graceful error handling and fallback mechanisms
- ✅ Production-ready application patterns

**The demo works with or without NATS server running**, demonstrating the robust fallback capabilities of the system.
