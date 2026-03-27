# BDD Specifications

## Feature: Binary-first installation

### Scenario: User installs MatrixClaw without privileged writes

Given a Linux or macOS machine without MatrixClaw installed  
And the user has a writable home directory  
When the user runs the install command  
Then the installer places the binary in a user-owned directory  
And the installer does not require Bun, Node.js, or Docker  
And the user can run `matrixclaw version` successfully

### Scenario: First launch opens setup without prior manual configuration

Given MatrixClaw is installed  
And no configuration file exists  
When the user runs `matrixclaw`  
Then MatrixClaw starts a local setup experience  
And the user can configure provider, workspace, and auth settings  
And MatrixClaw writes the resulting configuration to its home directory

## Feature: Streaming-first agent loop

### Scenario: Final assistant answer is generated once

Given an initialized session with no pending tool calls  
When the user sends a prompt  
Then MatrixClaw streams the assistant response from a single generation pass  
And the final persisted assistant message matches the streamed content exactly  
And the runtime does not perform a second completion just to enable streaming

### Scenario: Tool calls extend the turn loop

Given the model responds with one or more tool calls  
When MatrixClaw validates and executes those tool calls  
Then tool execution lifecycle events are emitted in order  
And tool result messages are appended to the session  
And the loop continues with the assistant using those tool results

### Scenario: Tool validation can block unsafe execution

Given a tool call is emitted by the model  
And a policy or hook determines the tool should not run  
When the tool preflight step executes  
Then the tool is blocked before invocation  
And a tool result message describing the block is emitted  
And the loop continues without crashing

## Feature: Queued message handling

### Scenario: Steering message is delivered before the next assistant turn

Given the assistant is currently processing a task with tool calls  
When the user queues a steering message  
Then MatrixClaw stores the steering message in the session runtime queue  
And delivers it before the next LLM turn begins  
And preserves the original ordering of prior tool results

### Scenario: Follow-up message is delivered only after the current run completes

Given the assistant is currently processing a task  
When the user queues a follow-up message  
Then MatrixClaw does not inject it into the current turn  
And delivers it only after the agent would otherwise stop

## Feature: Session persistence

### Scenario: Stored transcript matches visible behavior

Given a user has completed a conversation with tool calls, retries, and warnings  
When MatrixClaw persists the session  
Then every user-visible assistant message is present in session storage  
And every tool result that influenced the assistant is present in session storage  
And terminal warning or failure messages are also persisted when shown to the user

### Scenario: Session resumes after restart

Given an existing persisted session  
When MatrixClaw restarts  
Then the session runtime reloads the prior message history  
And the next prompt continues from the persisted state  
And the runtime can reconstruct queued metadata needed for further processing

## Feature: Compaction and retry outside the core loop

### Scenario: Runtime compacts context before retrying an overflowed request

Given a model request fails with context overflow  
When the session runtime handles the failure  
Then it removes the failure-only message from active context if needed  
And runs compaction outside the core loop  
And retries the run once from the compacted context

### Scenario: Compaction preserves role semantics

Given MatrixClaw compacts old conversation state  
When it inserts a summary artifact into active context  
Then that artifact is represented as a system or runtime summary message  
And it is not persisted as a user-authored message  
And the full pre-compaction history remains recoverable

## Feature: OpenClaw compatibility boundary

### Scenario: OpenClaw-compatible client lists agents

Given MatrixClaw is running with compatibility mode enabled  
When a compatible client authenticates over the OpenClaw WebSocket boundary  
Then the client receives the expected connection challenge and response flow  
And the client can request the list of available agents

### Scenario: OpenClaw-compatible chat request reaches the internal runtime

Given an authenticated compatibility client  
When it sends a chat request through the compatibility boundary  
Then MatrixClaw translates that request into internal session-runtime messages  
And the core loop runs without awareness of the external protocol format  
And the resulting events are translated back into compatibility responses

## Feature: OpenClaw ecosystem compatibility

### Scenario: User installs a text skill originally built for OpenClaw

Given the user has an OpenClaw-style markdown or text skill artifact  
When the user installs that skill into MatrixClaw  
Then MatrixClaw recognizes and imports the skill metadata  
And the skill becomes available to the runtime without requiring Node.js or Bun  
And the runtime records the artifact as an imported compatibility skill

### Scenario: User installs a subprocess-compatible plugin originally built for OpenClaw

Given the user has an OpenClaw plugin that communicates through a stable subprocess protocol  
When the user installs the plugin into MatrixClaw  
Then MatrixClaw classifies it as shim-compatible  
And launches it through the appropriate adapter layer  
And exposes the plugin capabilities to the runtime as native tool or channel abstractions

### Scenario: User tries to install an in-process OpenClaw extension tied to JS internals

Given the user has an OpenClaw extension that depends on in-process TypeScript or Bun runtime APIs  
When the user attempts to install it into MatrixClaw  
Then MatrixClaw refuses native installation  
And explains that this artifact requires a bridge runtime or manual rewrite  
And does not claim partial compatibility silently

## Feature: Managed optional assets

### Scenario: Browser engine downloads only on first use

Given MatrixClaw is installed without a browser engine asset  
When the user first invokes a browser-dependent capability  
Then MatrixClaw downloads the required asset into managed storage  
And subsequent browser requests reuse the installed asset  
And core chat and tool functionality still works without that asset before first use

## Feature: Sandbox behavior

### Scenario: Safe local execution works without Docker

Given the user has not installed Docker  
When the assistant uses local command execution  
Then MatrixClaw uses the default local execution mode  
And the runtime remains functional without failing startup

### Scenario: Optional sandbox mode is enabled explicitly

Given the user enables sandboxed execution in configuration  
When a tool requires isolated command execution  
Then MatrixClaw routes that command through the configured sandbox backend  
And returns structured execution results to the agent loop
