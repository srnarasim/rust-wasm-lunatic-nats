# System Architecture

## Overview

This project implements a scalable, robust framework for distributed, concurrent AI agents using:

**Rust**: for safe, high-performance agent logic and system code.

**WASM (WebAssembly)**: for secure, portable runtime environments for each agent.

**Lunatic**: an Erlang-inspired, actor-based runtime for orchestrating millions of lightweight, isolated WASM processes.

**NATS**: a high-performance, distributed messaging system with dual support for native TCP and WebSocket protocols, enabling both traditional server environments and WASM/browser deployments.

The design supports large-scale, fault-tolerant swarms of AI agents that can be orchestrated locally or across networks, with robust message passing, multi-target deployment capabilities, and easy extensibility for new agent behaviors, memory models, and deployment topologies.

## Key Concepts

### Agents
- Each agent is a Wasm process managed by Lunatic; implemented in idiomatic Rust.
- Agents are isolated ("no shared heap"), communicate only via asynchronous message-passing.
- Agents can be supervised and restarted by parent processes (Erlang "let it crash" philosophy).

### Supervisor
- Top-level controller responsible for spawning, monitoring, and restarting child agent processes.
- Implements agent pooling, task routing, health checking, and fault recovery.
- Uses Lunatic's supervisor trees for maximum reliability.

### Multi-Modal Communication
- **Lunatic Mailbox**: used for intra-node message passing (fast local channels between processes).
- **Native NATS (TCP)**: used for inter-node, inter-process, or remote/cloud agent messaging in server environments.
- **WebSocket NATS**: enables NATS messaging in WASM environments and browser deployments.
- **Subject-Based Routing**: supports pub/sub, queue groups, topic- and agent-based addressing.
- **Optional JetStream**: enables persistent event streams and message replay (planned).

### Memory & State
- **Ephemeral Context**: each agent maintains local state in its Wasm memory—fast, but volatile.
- **Long-Term/Persistent Context**: all important state is stored externally (database, key-value store, file system, or custom backends) via async-compatible APIs.
- **Dual-Layer Architecture**: automatic synchronization between ephemeral and persistent storage.
- Agents load relevant state on start/restart and never assume in-memory context will persist after failure or migration.

## Multi-Target Architecture

### Build Configurations

```mermaid
graph TB
    subgraph "Build Targets"
        subgraph "Native Builds"
            NATIVE[Native Development<br/>Full NATS TCP]
        end
        
        subgraph "WASM Builds"
            WASM_ONLY[WASM-Only<br/>Local Messaging]
            WASM_NATS[WASM + WebSocket NATS<br/>Distributed Messaging]
        end
    end
    
    subgraph "Feature Matrix"
        F1[default = nats]
        F2[wasm-only = []]
        F3[wasm-nats = websocket support]
        F4[nats = TCP client]
    end
    
    NATIVE --> F1
    NATIVE --> F4
    WASM_ONLY --> F2
    WASM_NATS --> F2
    WASM_NATS --> F3
    
    subgraph "Deployment Environments"
        ENV1[Server/Cloud<br/>Native Rust]
        ENV2[Lunatic Runtime<br/>Local WASM]
        ENV3[Browser/Edge<br/>WebSocket WASM]
    end
    
    NATIVE --> ENV1
    WASM_ONLY --> ENV2
    WASM_NATS --> ENV2
    WASM_NATS --> ENV3
```

## Recommended Practices
- Fail fast, retry safely: design supervisors to restart agents on error, restoring state from the persistent memory layer.
- Keep agents single-purpose: one agent = one main responsibility; chain/compose for complex workflows.
- Decouple via pub/sub: favor message-driven, decoupled logic for maximum resilience.
- Choose appropriate messaging: Lunatic mailboxes for local, NATS for distributed communication.
- Document all interfaces: schemas, expected message patterns, and storage contracts.

## Security Considerations
- Never expose internal NATS topics or agent APIs to untrusted networks.
- Use TLS for WebSocket NATS connections in production.
- Validate and sanitize all external message payloads.
- Never store secrets in code or unprotected config files.
- Leverage WASM sandboxing for agent isolation.

## High-Level System Architecture

```mermaid
graph TB
    subgraph "Development & Build System"
        BUILD_NATIVE[Native Build<br/>cargo build]
        BUILD_WASM[WASM Build<br/>cargo build --target=wasm32-wasip1]
        BUILD_WASM_NATS[WASM+NATS Build<br/>--features wasm-nats]
    end
    
    subgraph "Runtime Environment"
        subgraph "Lunatic Runtime (WASM)"
            subgraph "Main Supervisor Process"
                MAIN[Main Application]
                SUP[Supervisor Controller]
                MAIN --> SUP
            end
            
            subgraph "Agent Process Pool"
                A1[Agent A<br/>WASM Process]
                A2[Agent B<br/>WASM Process] 
                A3[Agent C<br/>WASM Process]
                AN[Agent N<br/>WASM Process]
                
                SUP -.->|spawn/monitor/restart| A1
                SUP -.->|spawn/monitor/restart| A2
                SUP -.->|spawn/monitor/restart| A3
                SUP -.->|spawn/monitor/restart| AN
            end
        end
    end
    
    subgraph "Communication Layer"
        subgraph "Local Communication"
            MAILBOX[Lunatic Mailboxes<br/>Process-to-Process<br/>Fast & Reliable]
        end
        
        subgraph "Distributed Communication"
            subgraph "Native NATS (TCP)"
                NATS_TCP[async-nats Client<br/>High Performance<br/>Full Feature Set]
                TCP_FEATURES[• Pub/Sub<br/>• Request/Reply<br/>• Queue Groups<br/>• JetStream Ready]
                NATS_TCP --> TCP_FEATURES
            end
            
            subgraph "WebSocket NATS (WASM)"
                NATS_WS[WebSocket Client<br/>Browser Compatible<br/>Protocol Compliant]
                WS_FEATURES[• Binary Protocol<br/>• Pub/Sub Support<br/>• Auto Reconnection<br/>• Gateway Integration]
                NATS_WS --> WS_FEATURES
            end
            
            subgraph "Gateway Layer"
                WS_GATEWAY[WebSocket Gateway<br/>Protocol Translation<br/>TLS Termination]
                NATS_WS -.->|WebSocket Frames| WS_GATEWAY
                WS_GATEWAY -.->|NATS Protocol| NATS_SERVER
            end
        end
    end
    
    subgraph "NATS Infrastructure"
        NATS_SERVER[NATS Server<br/>Subject Routing<br/>Clustering Support]
        NATS_TCP -.->|Direct TCP| NATS_SERVER
    end
    
    subgraph "State Management"
        subgraph "Ephemeral State (Process Memory)"
            ES1[Agent A Memory<br/>HashMap Cache]
            ES2[Agent B Memory<br/>HashMap Cache]
            ES3[Agent C Memory<br/>HashMap Cache]
            ESN[Agent N Memory<br/>HashMap Cache]
        end
        
        subgraph "Persistent Backends"
            MEM_BACKEND[In-Memory Backend<br/>Development/Testing]
            FILE_BACKEND[File Backend<br/>JSON Persistence]
            CUSTOM_BACKEND[Custom Backends<br/>Database/Redis/etc.]
        end
    end
    
    A1 <--> MAILBOX
    A2 <--> MAILBOX
    A3 <--> MAILBOX
    AN <--> MAILBOX
    
    A1 -.->|distributed| NATS_TCP
    A1 -.->|WASM environment| NATS_WS
    A2 -.->|distributed| NATS_TCP
    A2 -.->|WASM environment| NATS_WS
    A3 -.->|distributed| NATS_TCP
    A3 -.->|WASM environment| NATS_WS
    AN -.->|distributed| NATS_TCP
    AN -.->|WASM environment| NATS_WS
    
    A1 --> ES1
    A2 --> ES2
    A3 --> ES3
    AN --> ESN
    
    ES1 -.->|sync| MEM_BACKEND
    ES1 -.->|persist| FILE_BACKEND
    ES1 -.->|custom| CUSTOM_BACKEND
    ES2 -.->|sync| MEM_BACKEND
    ES2 -.->|persist| FILE_BACKEND
    ES2 -.->|custom| CUSTOM_BACKEND
    ES3 -.->|sync| MEM_BACKEND
    ES3 -.->|persist| FILE_BACKEND
    ES3 -.->|custom| CUSTOM_BACKEND
    ESN -.->|sync| MEM_BACKEND
    ESN -.->|persist| FILE_BACKEND
    ESN -.->|custom| CUSTOM_BACKEND
```

## Build Configuration Matrix

### Feature Flags & Targets

| Configuration | Features | Target | Use Case | NATS Support |
|---------------|----------|--------|----------|--------------|
| **Native Development** | `--features nats` | `x86_64/aarch64` | Development, Testing | Full TCP NATS |
| **WASM Local** | `--features wasm-only` | `wasm32-wasip1` | Lunatic Runtime | Local messaging only |
| **WASM Distributed** | `--features "wasm-only,wasm-nats"` | `wasm32-wasip1` | Browser, Edge Computing | WebSocket NATS |
| **Production Native** | `--features nats --release` | `x86_64/aarch64` | Server Deployment | Full TCP NATS |

### Build Commands Reference

```bash
# Development builds
cargo build                                    # Native with full NATS
cargo run                                      # Demo application

# WASM builds  
cargo build --target=wasm32-wasip1 --no-default-features --features wasm-only
cargo build --target=wasm32-wasip1 --no-default-features --features "wasm-only,wasm-nats"

# Testing matrix
cargo test                                     # Native tests
cargo test --no-default-features --features wasm-only              # WASM-only
cargo test --no-default-features --features "wasm-only,wasm-nats"  # WASM+WebSocket

# Production builds
cargo build --release --features nats         # Native production
cargo build --release --target=wasm32-wasip1 --no-default-features --features "wasm-only,wasm-nats"
```

## Core Components

### 1. Agent System (`src/agent.rs`)

The Agent is the fundamental unit of computation in the system, with dual representations for flexibility.

#### Dual Agent Architecture
```rust
// Lightweight handle for external API
pub struct Agent {
    pub id: AgentId,
}

// Full agent state for process execution
pub struct AgentState {
    pub id: AgentId,
    pub ephemeral_state: HashMap<String, serde_json::Value>,
    pub persistent_backend: Box<dyn MemoryBackend>,
    pub nats: Option<NatsConnection>,
}
```

#### Message Processing Flow
```mermaid
sequenceDiagram
    participant External as External Message
    participant Agent as Agent Process
    participant State as Ephemeral State
    participant Backend as Persistent Backend
    participant NATS as NATS (Optional)
    
    External->>+Agent: Receive Message
    Agent->>Agent: Parse Message Type
    
    alt State Action
        Agent->>+State: Update HashMap
        State->>+Backend: Persist Changes
        Backend-->>-State: Confirm Storage
        State-->>-Agent: State Updated
    else Business Logic
        Agent->>Agent: Process Application Logic
        Agent->>+State: Read/Write State
        State-->>-Agent: State Access
    else Forward Message
        Agent->>+NATS: Publish to Remote Agent
        NATS-->>-Agent: Message Sent
    end
    
    Agent-->>-External: Processing Complete
```

### 2. Lunatic Supervisor System (`src/supervisor.rs`)

Implements the actor supervisor pattern using Lunatic's process management.

#### Supervisor Architecture
```mermaid
classDiagram
    class AgentSupervisor {
        +Vec~AgentConfig~ configs
        +init(config, configs)
        +handle_failure(pid, reason)
        +restart_strategy() SupervisorStrategy
    }
    
    class AgentProcess {
        +AgentId id
        +HashMap~String,Value~ state
        +u32 message_count
        +init(config, arg) Result~Self~
        +terminate(state)
    }
    
    class AbstractProcess {
        <<interface>>
        +type Arg
        +type State
        +type Serializer
        +type Handlers
        +init(config, arg) Result~State~
        +terminate(state)
    }
    
    class Supervisor {
        <<interface>>
        +type Arg
        +type Children
        +init(config, arg)
        +strategy() SupervisorStrategy
    }
    
    AgentSupervisor ..|> Supervisor
    AgentProcess ..|> AbstractProcess
    AgentSupervisor --> AgentProcess : spawns/monitors
```

#### Supervisor API
```rust
// Spawn supervisor with multiple agent configurations
pub fn spawn_agent_supervisor(configs: Vec<AgentConfig>) -> Result<ProcessRef<AgentSupervisor>>;

// Spawn individual agent process
pub fn spawn_single_agent(config: AgentConfig) -> Result<ProcessRef<AgentProcess>>;

// Agent communication functions
pub fn send_message_to_agent(agent: &ProcessRef<AgentProcess>, message: Message);
pub fn send_state_action_to_agent(agent: &ProcessRef<AgentProcess>, action: StateAction);
pub fn get_agent_state(agent: &ProcessRef<AgentProcess>) -> HashMap<String, serde_json::Value>;
pub fn shutdown_agent(agent: &ProcessRef<AgentProcess>);
```

### 3. Dual NATS Communication Layer

#### 3.1 Native NATS (`src/nats_comm.rs`)

Built on the modern `async-nats` client providing full-featured messaging for server environments.

```mermaid
classDiagram
    class NatsConnection {
        -Client client
        -NatsConfig config
        +new(config) Result~Self~
        +publish(subject, data) Result~void~
        +subscribe(subject) Result~Vec~Message~~
        +request(subject, data) Result~Response~
        +get_stats() ConnectionStats
        +is_connected() bool
    }
    
    class NatsConfig {
        +String url
        +Duration timeout
        +Option~usize~ max_reconnects
        +Duration reconnect_delay
    }
    
    class ConnectionStats {
        +u64 messages_sent
        +u64 messages_received
        +u64 bytes_sent
        +u64 bytes_received
        +u64 reconnects
    }
    
    NatsConnection --> NatsConfig
    NatsConnection --> ConnectionStats
```

#### 3.2 WebSocket NATS (`src/wasm_nats.rs`)

WebSocket-based NATS client for WASM environments, maintaining protocol compatibility.

```mermaid
classDiagram
    class WasmNatsConnection {
        -WebSocket websocket
        -WasmNatsConfig config
        -Arc~Mutex~subscriptions~~ subscriptions
        -Arc~Mutex~bool~~ is_connected
        +new(config) Result~Self~
        +publish(subject, data) Result~void~
        +subscribe(subject) Result~UnboundedReceiver~Message~~
        +is_connected() bool
        +get_stats() WasmConnectionStats
    }
    
    class WasmNatsConfig {
        +String websocket_url
        +Duration timeout
        +Option~usize~ max_reconnects
        +Duration reconnect_delay
    }
    
    class WasmConnectionStats {
        +bool is_connected
        +u16 ready_state
        +String url
    }
    
    WasmNatsConnection --> WasmNatsConfig
    WasmNatsConnection --> WasmConnectionStats
```

#### Communication Patterns Comparison

| Pattern | Native NATS | WebSocket NATS | Use Case |
|---------|-------------|----------------|----------|
| **Publish/Subscribe** | ✅ Full Support | ✅ Full Support | Event Broadcasting |
| **Request/Reply** | ✅ Native Support | 🔄 Planned | Synchronous Communication |
| **Queue Groups** | ✅ Native Support | 🔄 Planned | Load Balancing |
| **Streams** | ✅ JetStream Ready | 🔄 Future | Persistent Messaging |
| **Binary Protocol** | ✅ Direct TCP | ✅ WebSocket Frames | Protocol Efficiency |
| **Browser Support** | ❌ Not Available | ✅ Full Support | Client Applications |

### 4. Memory Subsystem (`src/memory.rs`)

Enhanced pluggable storage abstraction supporting multiple backends with automatic synchronization.

#### Memory Backend Architecture
```mermaid
classDiagram
    class MemoryBackend {
        <<interface>>
        +store(key, value) Result~void~
        +retrieve(key) Result~Option~Value~~
        +delete(key) Result~bool~
        +list_keys(prefix) Result~Vec~String~~
        +clear() Result~void~
    }
    
    class InMemoryBackend {
        -Arc~Mutex~HashMap~~ storage
        +new() Self
        +with_capacity(capacity) Self
        +size() usize
    }
    
    class FileBackend {
        -PathBuf base_path
        -InMemoryBackend cache
        +new(path) Result~Self~
        +with_cache_size(size) Self
        -load_from_disk() Result~void~
        -save_to_disk(key, value) Result~void~
        -sync_cache() Result~void~
    }
    
    MemoryBackend <|.. InMemoryBackend
    MemoryBackend <|.. FileBackend
    FileBackend --> InMemoryBackend : uses as cache
```

#### Dual-Layer State Management

```mermaid
flowchart TD
    A[State Request] --> B{Check Agent Process<br/>Ephemeral State}
    B -->|Hit| C[Return from HashMap<br/>~1ms]
    B -->|Miss| D[Query Persistent Backend]
    
    D --> E{Backend Type}
    E -->|InMemory| F[Memory Lookup<br/>~1ms]
    E -->|File| G[File Read<br/>~10ms]
    E -->|Custom| H[Custom Backend<br/>Variable]
    
    F --> I[Cache in Process Memory]
    G --> I
    H --> I
    I --> J[Return Value]
    
    K[State Update] --> L[Update Process Memory<br/>Immediate]
    L --> M[Async Persist<br/>Background]
    
    M --> N{Persistence Strategy}
    N -->|Immediate| O[Sync Write]
    N -->|Batch| P[Queue for Batch Write]
    N -->|Interval| Q[Schedule Periodic Sync]
    
    R[Process Restart] --> S[Load All Persistent State]
    S --> T[Populate Process Memory]
    T --> U[Resume Normal Operation]
```

### 5. WebSocket Gateway Integration

For WASM deployments requiring NATS connectivity, a WebSocket gateway handles protocol translation.

#### Gateway Architecture Options

```mermaid
graph TB
    subgraph "WASM Client Environment"
        CLIENT[WASM Application<br/>with WebSocket NATS]
    end
    
    subgraph "Gateway Layer (Choose One)"
        subgraph "Option 1: Native NATS WebSocket"
            NATS_NATIVE[NATS Server v2.2+<br/>Built-in WebSocket Support]
        end
        
        subgraph "Option 2: Third-Party Gateway"
            THIRD_PARTY[nats-websocket-gw<br/>Protocol Translation]
        end
        
        subgraph "Option 3: Custom Gateway"
            CUSTOM[Custom Go/Rust Gateway<br/>TLS + Load Balancing]
        end
    end
    
    subgraph "NATS Infrastructure"
        NATS_CORE[NATS Server Core<br/>Subject Routing & Clustering]
    end
    
    CLIENT -.->|WebSocket (Binary)| NATS_NATIVE
    CLIENT -.->|WebSocket (Binary)| THIRD_PARTY
    CLIENT -.->|WebSocket (Binary)| CUSTOM
    
    NATS_NATIVE -.->|NATS Protocol| NATS_CORE
    THIRD_PARTY -.->|NATS Protocol| NATS_CORE
    CUSTOM -.->|NATS Protocol| NATS_CORE
```

## Data Flow Architectures

### 1. Multi-Configuration Message Flow

```mermaid
flowchart TD
    A[Incoming Message] --> B{Determine Message Destination}
    
    B -->|Local Agent| C[Lunatic Mailbox Routing]
    B -->|Remote Agent - Native| D[Native NATS TCP]
    B -->|Remote Agent - WASM| E[WebSocket NATS]
    
    C --> F[Agent Process Mailbox]
    D --> G[NATS Server TCP]
    E --> H[WebSocket Gateway]
    
    H --> I[Protocol Translation]
    I --> G
    G --> J[Subject-Based Routing]
    J --> K[Remote Node/Agent]
    
    F --> L[Agent Message Handler]
    K --> M[Remote Agent Handler]
    
    L --> N{Message Processing}
    M --> N
    
    N -->|State Update| O[Ephemeral State Cache]
    N -->|Business Logic| P[Application Processing]
    N -->|Forward/Reply| Q[Send Response]
    
    O --> R[Persistent Backend Sync]
    P --> S[State Updates]
    Q --> T{Response Destination}
    
    T -->|Local| C
    T -->|Remote Native| D
    T -->|Remote WASM| E
    
    S --> O
```

### 2. Fault-Tolerant Agent Lifecycle

```mermaid
stateDiagram-v2
    [*] --> Configured: Agent Configuration Created
    
    Configured --> Starting: Supervisor Spawn Request
    Starting --> StateLoading: Lunatic Process Created
    StateLoading --> Running: Persistent State Loaded
    
    state Running {
        [*] --> Processing
        Processing --> Processing: Handle Messages
        Processing --> StateUpdate: State Change
        StateUpdate --> PersistenceSync: Background Sync
        PersistenceSync --> Processing: Sync Complete
    }
    
    Running --> Crashed: Process Exception/Panic
    Running --> Stopping: Graceful Shutdown Request
    
    Crashed --> RestartEvaluation: Supervisor Notification
    state RestartEvaluation {
        [*] --> CheckPolicy
        CheckPolicy --> AllowRestart: Within Limits
        CheckPolicy --> PermanentFailure: Max Restarts Exceeded
    }
    
    AllowRestart --> Starting: Respawn Process
    PermanentFailure --> Failed: Agent Marked Failed
    
    Stopping --> StatePersistence: Save Ephemeral State
    StatePersistence --> Cleanup: State Saved
    Cleanup --> Terminated: Resources Released
    
    Failed --> [*]: Permanent Failure
    Terminated --> [*]: Clean Shutdown
```

### 3. Multi-Backend State Synchronization

```mermaid
sequenceDiagram
    participant App as Application
    participant Agent as Agent Process
    participant Cache as Ephemeral Cache
    participant Backend as Persistent Backend
    participant Gateway as WebSocket Gateway
    participant NATS as NATS Server
    
    Note over App, NATS: Agent Startup Sequence
    App->>+Agent: Initialize Agent
    Agent->>+Backend: Load Persistent State
    Backend-->>-Agent: Historical State Data
    Agent->>+Cache: Populate Process Memory
    Cache-->>-Agent: Cache Ready
    Agent-->>-App: Agent Initialized
    
    Note over App, NATS: Normal Operation
    App->>+Agent: Send State Update
    Agent->>+Cache: Update Immediate
    Cache-->>-Agent: Cache Updated
    Agent-->>-App: Immediate Response
    
    par Background Persistence
        Agent->>+Backend: Async Persist
        Backend-->>-Agent: Persistence Complete
    and Optional NATS Broadcast
        Agent->>+Gateway: State Change Event
        Gateway->>+NATS: Publish Update
        NATS-->>-Gateway: Event Published
        Gateway-->>-Agent: Broadcast Complete
    end
    
    Note over App, NATS: Agent Recovery
    Agent->>Agent: Process Crash
    App->>+Agent: Respawn Process
    Agent->>+Backend: Reload State
    Backend-->>-Agent: Latest Persisted State
    Agent->>+Cache: Rebuild Memory
    Cache-->>-Agent: Memory Restored
    Agent-->>-App: Recovery Complete
```

## Deployment Patterns

### 1. Single-Node Development

```mermaid
graph TB
    subgraph "Development Machine"
        subgraph "Lunatic Runtime"
            DEV_MAIN[Main Process]
            DEV_SUP[Supervisor]
            DEV_A1[Agent 1]
            DEV_A2[Agent 2]
            DEV_A3[Agent N]
            
            DEV_MAIN --> DEV_SUP
            DEV_SUP -.-> DEV_A1
            DEV_SUP -.-> DEV_A2
            DEV_SUP -.-> DEV_A3
        end
        
        subgraph "Local Services"
            DEV_NATS[NATS Server<br/>localhost:4222]
            DEV_STORAGE[Local File Storage<br/>./data/]
        end
        
        DEV_A1 <--> DEV_NATS
        DEV_A2 <--> DEV_NATS
        DEV_A3 <--> DEV_NATS
        
        DEV_A1 --> DEV_STORAGE
        DEV_A2 --> DEV_STORAGE
        DEV_A3 --> DEV_STORAGE
    end
```

### 2. Production Multi-Node Cluster

```mermaid
graph TB
    subgraph "Load Balancer"
        LB[HAProxy/nginx<br/>TLS Termination]
    end
    
    subgraph "Node A - Primary"
        subgraph "Lunatic Runtime A"
            A_MAIN[Main Process A]
            A_SUP[Supervisor A]
            A_AGENTS[Agent Pool A<br/>A1, A2, A3...]
            
            A_MAIN --> A_SUP
            A_SUP -.-> A_AGENTS
        end
        
        A_STORAGE[Persistent Storage A<br/>SSD/Database]
        A_AGENTS --> A_STORAGE
    end
    
    subgraph "Node B - Secondary"
        subgraph "Lunatic Runtime B"
            B_MAIN[Main Process B]
            B_SUP[Supervisor B]
            B_AGENTS[Agent Pool B<br/>B1, B2, B3...]
            
            B_MAIN --> B_SUP
            B_SUP -.-> B_AGENTS
        end
        
        B_STORAGE[Persistent Storage B<br/>SSD/Database]
        B_AGENTS --> B_STORAGE
    end
    
    subgraph "Node C - Worker"
        subgraph "Lunatic Runtime C"
            C_MAIN[Main Process C]
            C_SUP[Supervisor C]
            C_AGENTS[Agent Pool C<br/>C1, C2, C3...]
            
            C_MAIN --> C_SUP
            C_SUP -.-> C_AGENTS
        end
        
        C_STORAGE[Persistent Storage C<br/>SSD/Database]
        C_AGENTS --> C_STORAGE
    end
    
    subgraph "NATS Cluster"
        N1[NATS Node 1<br/>Primary]
        N2[NATS Node 2<br/>Secondary]
        N3[NATS Node 3<br/>Worker]
        
        N1 --- N2
        N2 --- N3
        N3 --- N1
    end
    
    subgraph "Monitoring Stack"
        METRICS[Prometheus<br/>Metrics Collection]
        LOGS[Grafana<br/>Dashboards & Alerts]
        TRACE[Jaeger<br/>Distributed Tracing]
    end
    
    LB --> A_MAIN
    LB --> B_MAIN
    LB --> C_MAIN
    
    A_AGENTS -.->|distributed| N1
    B_AGENTS -.->|distributed| N2
    C_AGENTS -.->|distributed| N3
    
    A_AGENTS -.-> METRICS
    B_AGENTS -.-> METRICS
    C_AGENTS -.-> METRICS
    
    METRICS --> LOGS
    METRICS --> TRACE
```

### 3. Hybrid Cloud-Edge Deployment

```mermaid
graph TB
    subgraph "Cloud Infrastructure"
        subgraph "Control Plane"
            CLOUD_LB[Cloud Load Balancer]
            CLOUD_NATS[NATS Cluster<br/>3+ Nodes]
            CLOUD_DB[(Distributed Database<br/>PostgreSQL/CockroachDB)]
        end
        
        subgraph "Cloud Worker Nodes"
            CLOUD_A[Cloud Node A<br/>High-Memory Agents]
            CLOUD_B[Cloud Node B<br/>Compute-Intensive Agents]
            CLOUD_C[Cloud Node C<br/>Coordination Agents]
        end
    end
    
    subgraph "Edge Locations"
        subgraph "Edge Site 1"
            EDGE1_NODE[Lunatic Runtime<br/>Local Agents]
            EDGE1_CACHE[Local Cache<br/>Redis/Files]
            EDGE1_GW[WebSocket Gateway<br/>TLS Proxy]
        end
        
        subgraph "Edge Site 2"
            EDGE2_NODE[Lunatic Runtime<br/>Local Agents]
            EDGE2_CACHE[Local Cache<br/>Redis/Files]
            EDGE2_GW[WebSocket Gateway<br/>TLS Proxy]
        end
    end
    
    subgraph "Browser/Mobile Clients"
        CLIENT1[WASM App 1<br/>WebSocket NATS]
        CLIENT2[WASM App 2<br/>WebSocket NATS]
        CLIENT3[Mobile App<br/>HTTP/WebSocket]
    end
    
    CLOUD_LB --> CLOUD_A
    CLOUD_LB --> CLOUD_B
    CLOUD_LB --> CLOUD_C
    
    CLOUD_A -.->|native TCP| CLOUD_NATS
    CLOUD_B -.->|native TCP| CLOUD_NATS
    CLOUD_C -.->|native TCP| CLOUD_NATS
    
    CLOUD_A --> CLOUD_DB
    CLOUD_B --> CLOUD_DB
    CLOUD_C --> CLOUD_DB
    
    EDGE1_GW -.->|secure tunnel| CLOUD_NATS
    EDGE2_GW -.->|secure tunnel| CLOUD_NATS
    
    EDGE1_NODE -.->|WebSocket| EDGE1_GW
    EDGE2_NODE -.->|WebSocket| EDGE2_GW
    
    EDGE1_NODE --> EDGE1_CACHE
    EDGE2_NODE --> EDGE2_CACHE
    
    CLIENT1 -.->|WebSocket/TLS| EDGE1_GW
    CLIENT2 -.->|WebSocket/TLS| EDGE2_GW
    CLIENT3 -.->|HTTP/WebSocket| CLOUD_LB
```

## Performance Characteristics & Optimizations

### 1. Performance Benchmarks

| Metric | Local Lunatic | Native NATS | WebSocket NATS | File Backend |
|--------|---------------|-------------|-----------------|--------------|
| **Latency (p50)** | <1ms | 2-5ms | 5-15ms | 10-50ms |
| **Latency (p99)** | 2ms | 10-20ms | 25-50ms | 100-200ms |
| **Throughput** | 100K+ ops/sec | 50K+ msgs/sec | 5K+ msgs/sec | 1K+ ops/sec |
| **Memory/Agent** | 1-5KB | 2-10KB | 5-15KB | 10-100KB |
| **CPU Usage** | Low | Medium | Medium-High | Medium |

### 2. Scalability Patterns

```mermaid
graph TB
    subgraph "Vertical Scaling (Single Node)"
        VS1[Small: 100 Agents<br/>1 CPU, 512MB RAM]
        VS2[Medium: 1K Agents<br/>4 CPU, 2GB RAM]
        VS3[Large: 10K Agents<br/>16 CPU, 8GB RAM]
        VS4[XLarge: 100K Agents<br/>64 CPU, 32GB RAM]
        
        VS1 --> VS2 --> VS3 --> VS4
    end
    
    subgraph "Horizontal Scaling (Multi-Node)"
        HS1[2 Nodes<br/>2x Capacity]
        HS2[5 Nodes<br/>5x Capacity]
        HS3[10+ Nodes<br/>10x+ Capacity]
        
        HS1 --> HS2 --> HS3
    end
    
    subgraph "Geographic Distribution"
        GEO1[Region A<br/>Low Latency Users]
        GEO2[Region B<br/>Medium Latency Users]
        GEO3[Edge Sites<br/>Ultra-Low Latency]
        
        GEO1 -.-> GEO2
        GEO2 -.-> GEO3
    end
    
    VS4 --> HS1
    HS3 --> GEO1
```

### 3. Optimization Strategies

#### Message Processing Optimization
```rust
// Batch message processing
impl Agent {
    async fn process_message_batch(&mut self, messages: Vec<Message>) -> Result<()> {
        // Group messages by type
        let mut state_updates = Vec::new();
        let mut business_logic = Vec::new();
        
        for msg in messages {
            match msg.payload.get("type") {
                Some("state") => state_updates.push(msg),
                _ => business_logic.push(msg),
            }
        }
        
        // Batch process state updates
        self.process_state_batch(state_updates).await?;
        
        // Process business logic concurrently
        let futures = business_logic.into_iter()
            .map(|msg| self.process_business_message(msg));
        futures::future::try_join_all(futures).await?;
        
        Ok(())
    }
}
```

## Security & Compliance

### 1. Security Layers

```mermaid
graph TB
    subgraph "Application Security"
        INPUT_VAL[Input Validation<br/>Message Sanitization]
        AUTH[Authentication<br/>Agent Identity]
        AUTHZ[Authorization<br/>Subject-Based ACL]
    end
    
    subgraph "Transport Security"
        TLS[TLS Encryption<br/>WebSocket/TCP]
        CERT[Certificate Management<br/>Mutual TLS]
        PROXY[Reverse Proxy<br/>Rate Limiting]
    end
    
    subgraph "Runtime Security"
        WASM_SANDBOX[WASM Sandboxing<br/>Memory Isolation]
        PROCESS_ISOLATION[Process Isolation<br/>Lunatic Boundaries]
        RESOURCE_LIMITS[Resource Limits<br/>CPU/Memory Quotas]
    end
    
    subgraph "Data Security"
        ENCRYPT_REST[Encryption at Rest<br/>File/Database Encryption]
        ENCRYPT_TRANSIT[Encryption in Transit<br/>All Network Traffic]
        KEY_MGMT[Key Management<br/>Rotation & Secrets]
    end
    
    INPUT_VAL --> AUTH --> AUTHZ
    TLS --> CERT --> PROXY
    WASM_SANDBOX --> PROCESS_ISOLATION --> RESOURCE_LIMITS
    ENCRYPT_REST --> ENCRYPT_TRANSIT --> KEY_MGMT
```

### 2. Compliance Features

| Requirement | Implementation | Status |
|-------------|----------------|--------|
| **Data Isolation** | WASM process sandboxing | ✅ Implemented |
| **Audit Logging** | Structured logging with serde | ✅ Implemented |
| **Access Control** | NATS subject-based permissions | 🔄 Configurable |
| **Encryption** | TLS for all network traffic | ✅ Implemented |
| **Key Rotation** | External key management integration | 🔄 Planned |
| **Compliance Reporting** | Metrics and audit trail export | 🔄 Planned |

## Future Enhancements & Roadmap

### Phase 1: Advanced Messaging (Q1-Q2)
- **JetStream Integration**: Persistent streams and message replay
- **Advanced WebSocket NATS**: Request/reply and queue group support
- **Message Compression**: Automatic compression for WebSocket transport
- **Connection Pooling**: Multi-connection support for high throughput

### Phase 2: Observability & Operations (Q2-Q3)
- **OpenTelemetry Integration**: Distributed tracing across agents
- **Prometheus Metrics**: Custom metrics collection and export
- **Health Checks**: Agent and system health monitoring
- **Configuration Management**: Dynamic configuration updates

### Phase 3: Production Features (Q3-Q4)
- **Agent Discovery**: Automatic service discovery via NATS
- **Load Balancing**: Intelligent request routing and agent placement
- **Multi-Tenancy**: Namespace isolation and resource quotas
- **Backup & Recovery**: State backup and point-in-time recovery

### Phase 4: Advanced Capabilities (Q4+)
- **Zero-Downtime Updates**: Rolling updates with state migration
- **Cross-Region Replication**: Multi-region state synchronization
- **AI/ML Integration**: Native support for AI model inference
- **Edge Computing**: Optimized builds for edge deployment

## Development Guidelines

### 1. Adding New Features

#### Multi-Target Compatibility Checklist
```rust
// Feature flag structure
#[cfg(feature = "nats")]          // Native NATS functionality
#[cfg(feature = "wasm-nats")]     // WebSocket NATS functionality  
#[cfg(not(feature = "nats"))]     // Stub implementations
#[cfg(target_arch = "wasm32")]    // WASM-specific code
#[cfg(not(target_arch = "wasm32"))] // Native-specific code
```

#### Agent Implementation Pattern
```rust
// 1. Define message types
#[derive(Serialize, Deserialize)]
pub struct CustomMessage {
    pub operation: String,
    pub data: serde_json::Value,
}

// 2. Implement message handler
impl Agent {
    pub async fn handle_custom_message(&mut self, msg: CustomMessage) -> Result<()> {
        match msg.operation.as_str() {
            "process" => self.process_data(msg.data).await,
            "store" => self.store_data(msg.data).await,
            _ => Err(Error::Custom("Unknown operation".to_string()))
        }
    }
}

// 3. Add NATS subject routing
const CUSTOM_SUBJECT: &str = "agents.custom.operations";

// 4. Add tests for all configurations
#[cfg(test)]
mod tests {
    #[test] fn test_native() { /* Native test */ }
    
    #[cfg(feature = "wasm-nats")]
    #[test] fn test_wasm_nats() { /* WASM+NATS test */ }
    
    #[cfg(not(feature = "nats"))]
    #[test] fn test_wasm_only() { /* WASM-only test */ }
}
```

### 2. Performance Testing Framework

```rust
#[cfg(test)]
mod benchmarks {
    use criterion::{criterion_main, criterion_group, Criterion};
    
    fn benchmark_agent_creation(c: &mut Criterion) {
        c.bench_function("agent_creation", |b| {
            b.iter(|| {
                // Benchmark agent creation across configurations
            });
        });
    }
    
    fn benchmark_message_throughput(c: &mut Criterion) {
        c.bench_function("message_throughput", |b| {
            b.iter(|| {
                // Benchmark message processing rates
            });
        });
    }
    
    criterion_group!(benches, benchmark_agent_creation, benchmark_message_throughput);
    criterion_main!(benches);
}
```

### 3. Documentation Standards

Every new feature must include:
- **API Documentation**: Comprehensive rustdoc with examples
- **Configuration Guide**: Feature flags and environment variables
- **Performance Impact**: Benchmarks and resource usage
- **Migration Guide**: Changes required for existing code
- **Security Considerations**: Security implications and mitigations

This architecture provides a comprehensive foundation for building scalable, distributed agent-based systems with support for multiple deployment targets, robust fault tolerance, and extensive messaging capabilities across native and WebAssembly environments.