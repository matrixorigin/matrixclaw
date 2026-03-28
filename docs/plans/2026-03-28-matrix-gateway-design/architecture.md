# Architecture

## Boundary

MatrixClaw should keep exactly one live runtime service and one persisted session model. External channels must not call the runtime with transport-specific request types. Instead, each gateway adapter should normalize inbound events into the existing ingress envelope and project outbound runtime events back into channel-specific deliveries.

## Layers

1. `gateway` layer
- owns connector-specific event parsing, auth/client calls, retry, dedupe, and reply routing
- converts inbound gateway traffic into normalized ingress requests
- converts runtime events and completions into gateway deliveries

2. `ingress` layer
- remains transport-neutral
- carries sender identity, channel/thread identity, target-agent identity, prompt payload, seed history, and reply-routing metadata
- invokes the shared live runtime

3. `live runtime`
- owns prompt projection, tool loop, session persistence, and queue semantics
- stays unaware of Matrix-specific event ids, retries, or client APIs

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
