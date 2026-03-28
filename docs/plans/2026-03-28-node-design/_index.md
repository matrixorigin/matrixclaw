# Node Design

**Goal:** Define the Node boundary as the host capability layer that sits below the live runtime, then choose the first concrete Node slice in a way that reuses the current execution modules instead of introducing a second capability system.

**Why now:** MatrixClaw now has the correct communication-side vocabulary and baseline implementation: browser, OpenClaw, and Matrix all fit the Gateway model. The next architectural risk is capability sprawl. Execution, sandboxing, local commands, plugin launching, and future screenshots/browser/device powers need one coherent model before more host abilities are added.

**Decision:** The first concrete Node slice should be execution-oriented, not browser/device-oriented.

That means:
- first Node boundary proves shell and sandbox execution
- screenshots, browser automation, camera, mouse, and similar powers can be added later as sibling Nodes
- the runtime gets one stable capability boundary instead of many unrelated helpers

**Design Files:**
- [Architecture](./architecture.md)
