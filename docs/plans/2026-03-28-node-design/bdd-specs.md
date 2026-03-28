# BDD Specs

## Feature: Execution Node contract

### Scenario: Runtime reaches execution through a Node boundary
Given the runtime needs to execute a host command
When it issues a request through the Execution Node contract
Then the request is represented as a Node-specific capability request
And the Node returns a structured capability result
And the runtime does not need to know local or sandbox backend implementation details

## Feature: Execution Node policy and routing

### Scenario: Execution Node routes local, sandboxed, and denied execution
Given execution policy may allow local execution, require sandboxing, or deny execution
When the Execution Node handles a capability request
Then it routes the request to the correct backend
And it reports the backend used in the structured result
And denied execution fails at the Node boundary rather than inside Gateway logic

## Feature: Runtime and tool reuse

### Scenario: Tool-backed runtime execution reuses the Execution Node
Given a runtime tool requires host command execution
When the tool is executed during a live runtime turn
Then the runtime reaches the host through the Execution Node boundary
And the resulting structured output is preserved in runtime-visible results
And existing Gateway behavior remains unchanged

## Feature: Verification and future sibling Nodes

### Scenario: Execution Node establishes the pattern for future Nodes
Given the Execution Node is the first concrete Node slice
When maintainers verify the milestone
Then focused tests and a smoke harness prove the Node boundary works end-to-end
And future Screenshot, Browser, Camera, Mouse, and Filesystem Nodes can follow the same layering
