# PRP: LLM-Guided Lunatic Agent for Cloud Browser Automation via BrowserBase

## The Value

Enable secure, scalable, and intelligent browser automation workflows orchestrated by AI agents:
- Agents run as sandboxed WASM actors (Lunatic) for isolation and concurrency.
- LLMs (local/cloud) provide reasoning, planning, and summarization.
- BrowserBase serves as a robust, cloud-first MCP browser automation backend.
- Ideal for research, monitoring, scraping, E2E testing, and agentic web workflows.

## The Scope

- Implement a sample agent system with these components:
  1. **Lunatic actor** (WASM/Rust) acting as agent/orchestrator.
  2. **Integration with an LLM** for planning (prompting, summarization, next-action selection).
  3. **HTTP API calls from Lunatic to BrowserBase** for browser control (open session, navigate, click, extract, screenshot, close).
  4. **Loop**: LLM decides next step based on user/task objective and results from the previous browser step.
  5. **Result aggregation** (collating and presenting agent actions + LLM reasoning as a report).

## Success Criteria

- One or more Lunatic actors can start a browser session in BrowserBase, direct actions, and end session.
- Agent(s) interact with an LLM to:
  - Plan the sequence of navigations/interactions.
  - Summarize findings from web automation.
- The workflow is modular and extensible (plug in LLM, swap orchestration flows).
- Works with real BrowserBase account/API keys and is easy to run/test.

## Context

- [BrowserBase API & MCP documentation](https://docs.browserbase.com/integrations/mcp/introduction)
- [Lunatic WASM runtime](https://lunatic.solutions/)
- [Example Lunatic agent + HTTP Rust libraries (`ureq`)](https://docs.rs/ureq/)
- [Calling OpenAI/Anthropic from Rust](https://docs.rs/ureq/latest/ureq/)
- [LLM orchestration pattern](https://github.com/langchain-ai/langchain)
- Example PRPs: [github.com/coleam00/context-engineering-intro](https://github.com/coleam00/context-engineering-intro)

## Implementation Blueprint

1. **Agent Initialization:**
   - Compile Rust agent to WASM; run using Lunatic runtime.
   - Read config: BrowserBase API key, LLM (API or subprocess), target query.
2. **Session Management:**
   - Start session with BrowserBase API (`POST /sessions`).
   - Store returned session ID.
3. **Task Loop:**
   - For each LLM-planned step:
     - Send relevant BrowserBase API call (navigate, click, etc.).
     - Receive and store browser output (DOM, screenshot, etc.).
     - Query LLM with output + next goal to get next action or terminate.
   - Repeat until LLM says stop or max steps reached.
4. **Result Handling:**
   - Upon workflow completion, collate all data.
   - Use LLM to produce summary/report if needed.
5. **Cleanup:**
   - Close BrowserBase session.

## API Contracts & Data Flow

- All browser actions via BrowserBase HTTP API per its documentation.
- LLM calls via HTTP (OpenAI/Anthropic) or subprocess bridge for local models.
- Data passed between Lunatic actor, BrowserBase, and LLM as JSON.

## Known Gotchas

- Handle API/network errors gracefully; plan for retries/timeouts.
- BrowserBase API key management: never hard-code in source, use environment variables.
- LLM latency may introduce step lag; consider async task orchestration for scale.
- Manage BrowserBase session limits and step quotas.

## Test Requirements

- Run end-to-end sample for simple workflows (e.g., “Go to github.com, search ‘rust mcp’, scrape repo names, summarize”).
- Test with multiple agents in parallel (Lunatic actors).
- Simulate network/API failures.

## Example Tasks

- User: “Summarize top articles on ‘WASM-based browser automation’ from Hacker News using automated browsing.”
- The agent and LLM will plan queries, navigate, extract, and summarize.

## Documentation & References

- BrowserBase API & tools: https://docs.browserbase.com/integrations/mcp/tools
- Lunatic actors & networking: https://lunatic.solutions/docs/net/
- Rust HTTP (`ureq`): https://docs.rs/ureq/
- OpenAI Chat API (for LLM): https://platform.openai.com/docs/api-reference/chat

## Other Considerations

- Ensure secure handling of API keys, session tokens, and sensitive data.
- Make orchestration modular to allow plugging in different LLM providers or agentic flows.
- Document clearly with sample config files and usage instructions.


