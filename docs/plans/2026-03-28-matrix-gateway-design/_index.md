# Matrix Gateway Design

**Goal:** Add a generic external gateway boundary on top of the new ingress contract, then prove it with a Matrix-first gateway that can receive room messages, reuse persisted sessions, and stream replies back without leaking Matrix-specific behavior into the runtime core.

**Why now:** The served-transport phase completed the internal shape we needed: one live runtime, one session store, one ingress normalization layer, and one repeatable served-transport smoke harness. The next useful step is to validate that this architecture actually scales to a real IM-style gateway instead of only browser and OpenClaw transports.

**Scope:**
- generic gateway adapter contract in `matrixclaw-app-host`
- Matrix-first inbound event normalization and outbound delivery projection
- gateway-local dedupe/retry state outside the runtime core
- optional startup/config wiring
- browser/Matrix shared-session smoke coverage

**Out of scope:**
- Slack, Telegram, Discord, or multi-gateway support
- full Matrix auth/device-management production hardening
- media upload/download support
- multi-user authorization redesign

**Design Files:**
- [BDD Specs](./bdd-specs.md)
- [Architecture](./architecture.md)
- [Gateway and Node Model](./gateway-and-node-model.md)
