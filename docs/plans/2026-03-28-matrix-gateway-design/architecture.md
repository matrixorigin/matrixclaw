# Architecture

## Terminology

- `Gateway`
  - the external messaging boundary
  - owns channel-specific ingress, egress, routing, retry, dedupe, and workspace/session mapping
- `Node`
  - the capability boundary between the runtime and host/system abilities
  - owns screenshots, browsing, camera, mouse, shell, filesystem, and similar powers
- `Ingress`
  - the normalized internal request envelope passed from a gateway into the runtime
- `Live runtime`
  - the shared execution core between gateways and nodes

## Boundary

MatrixClaw should keep exactly one live runtime service and one persisted session model. External channels must not call the runtime with transport-specific request types. Instead, each gateway adapter should normalize inbound events into the existing ingress envelope and project outbound runtime events back into channel-specific deliveries.

## Layers

1. `gateway` layer
- owns connector-specific event parsing, auth/client calls, retry, dedupe, reply routing, and workspace/session resolution
- converts inbound gateway traffic into normalized ingress requests
- converts runtime events and completions into gateway deliveries

2. `ingress` layer
- remains transport-neutral
- carries sender identity, channel/thread identity, target-agent identity, prompt payload, seed history, and reply-routing metadata
- invokes the shared live runtime

3. `live runtime`
- owns prompt projection, tool loop, session persistence, and queue semantics
- stays unaware of Matrix-specific event ids, retries, or client APIs

4. `node` layer
- owns capability-specific APIs into the host system
- receives runtime-issued actions and returns structured results
- stays unaware of Matrix, browser, or OpenClaw delivery semantics

## Gateway and Node split

Gateways are about communication.
Nodes are about powers.

That means:
- Matrix, OpenClaw, browser, Telegram, and future IM channels belong in `gateway`
- screenshots, browser automation, camera access, mouse movement, shell execution, and filesystem access belong in `node`

The runtime sits between them:
- gateways bring messages in and send replies out
- runtime plans and persists turns
- nodes execute host abilities on behalf of the runtime

## Matrix-first proving case

The first real gateway should be Matrix because the product direction already assumes room/thread semantics and the current ingress shape naturally fits sender/channel/thread routing. A successful Matrix-first adapter should make later gateways mostly an adapter exercise rather than another runtime rewrite.

## Storage split

- runtime session history remains under the existing session store
- Matrix room/thread to session mappings should live in gateway-owned persistence
- dedupe keys and retry state should also stay gateway-owned

## Validation rule

No gateway phase is complete until a smoke harness proves:
- inbound Matrix event -> normalized ingress -> shared runtime
- persisted browser session -> resumed by Matrix mapping
- streamed runtime output -> projected back to Matrix delivery

No node phase is complete until a smoke harness proves:
- a runtime-issued capability request reaches the intended host boundary
- capability-specific permission and policy checks happen outside the runtime core
- the resulting node output is returned to the runtime in a stable structured form
