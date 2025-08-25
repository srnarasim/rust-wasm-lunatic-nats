# PRP: LLM-Augmented Distributed Agent System on Rust WASM Lunatic NATS

## The Value

Build a scalable, resilient, and intelligent distributed agent system leveraging:
- Rust and WASM for secure, portable plugin development
- Lunatic runtime for Erlang-like process isolation, supervisor trees, and rapid actor-based concurrency
- NATS messaging for high-throughput, distributed agent communication (local and WebSocket for browser/WASM environments)
- Integration of LLMs for reasoning, summarization, and planning—agents become truly intelligent, not just reactive

## The Scope

- Create a distributed multi-agent system with these core capabilities:
  1. **Agent processes** run as isolated WASM instances under Lunatic, managed by supervisors
  2. **Messaging layer:** Dual mode—local mailboxes for intra-runtime, plus NATS for cluster-wide agent communication (TCP or WebSocket as needed)
  3. **State management:** In-memory or file-based persistence per agent, pluggable backend
  4. **LLM orchestration:** Agents invoke LLM APIs for task planning, adaptive reasoning, or intelligent summarization
  5. **Scalability and Fault Tolerance:** Supervisor tree manages agent lifecycles; failed agents restart automatically
  6. **Example use case:** Distributed agents search, scrape, summarize, or process data from multiple sources and coordinate via NATS

## Success Criteria

- Agents can be instantiated, supervised, and message-passed locally and across the network (NATS)
- State is managed per agent, persists as required
- LLM reasoning is included—agents use LLM calls for reasoning, summarization, planning workflow steps, or synthesizing distributed results
- Demo: Run parallel agents that collect data, route results to a summarizer agent, which invokes an LLM and publishes output to cluster

## Context

- [Project repository](https://github.com/srnarasim/rust-wasm-lunatic-nats)
- [Lunatic docs](https://lunatic.solutions/docs/)
- [NATS documentation](https://docs.nats.io/)
- Sample LLM APIs (OpenAI, Anthropic, etc.)  
  *Can be called from Rust agents using HTTP clients like `ureq`, or via Python as a subprocess/sidecar if desired*

## Implementation Blueprint

1. **Design:**
   - Supervisor tree configuration (main supervisor, agent child processes)
   - Pluggable state backend: ephemeral, file-based, or external store
   - Choose local mailbox vs. NATS message routing per communication case

2. **Agent workflow:**
   - On spawn: Initialize state, subscribe to mailbox/NATS topic
   - On message: Perform action (e.g., data processing, crawling), optionally call LLM for reasoning (“What should I do next given current data?”)
   - Publish results (e.g., via NATS, to storage, or to next agent)

3. **LLM integration:**
   - Sample agent logic:
     - Receive set of task results
     - Compose LLM prompt and send to API (e.g., summarize findings, generate insight, decide next step)
   - Handle responses, update state, publish results

4. **Distributed messaging:**
   - Use async-nats (TCP) or ws_stream_wasm (WebSocket) for NATS client
   - Each agent subscribes/publishes on its subject
   - Cluster can span native and WASM (browser) runtimes

5. **Fault tolerance:**
   - If an agent panics/crashes, supervisor auto-restarts with fresh state
   - All messaging and persistent state survives agent failures

## Known Gotchas

- Ensure WASM/WASI networking permissions for NATS connectivity
- Secure API keys for LLM providers
- Network partitions: NATS auto-reconnect and robust event handling
- LLM latency can introduce bottlenecks—consider async job queues or batching

## Test Requirements

- Run agents that:
  - Distribute tasks via NATS, report results to summarizing agent
  - Survive random failures (simulate crashes, auto-restart)
  - Scale horizontally—hundreds of agents, multiple runtimes
  - Demonstrate LLM-driven result synthesis (real or mocked API)

## Example Use Case

- **Distributed Web Scraping:** Agents crawl assigned URLs, publish content via NATS, summarizer agent collects, calls LLM to produce insight, and publishes to results topic
- **Collaborative Monitoring:** Multiple agents track sources (site, API, logs); aggregate and summarize via LLM agent

## Documentation & Reference Links

- [rust-wasm-lunatic-nats README](https://github.com/srnarasim/rust-wasm-lunatic-nats)
- [Lunatic Supervisor Pattern](https://lunatic.solutions/docs/supervisor/)
- [NATS — Distributed Messaging](https://docs.nats.io/)
- [OpenAI API guide](https://platform.openai.com/docs/api-reference/chat)
- [Sample multi-agent demo architecture](https://lunatic.solutions/blog/writing-rust-the-elixir-way-1.5-years-later/)

## Other Considerations

- Modular design allows swapping memory backend, LLM providers, messaging transports
- Ensure compatibility with browser environments via WebSocket NATS
- Document sample flows, agent configs, and LLM prompt patterns

