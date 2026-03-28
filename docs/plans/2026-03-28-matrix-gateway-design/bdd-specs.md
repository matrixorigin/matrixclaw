# BDD Specifications

## Feature: Gateway adapter boundary

### Scenario: External channel event is normalized before entering the runtime

Given an inbound message arrives from an external IM gateway  
And the gateway carries sender, channel, thread, and reply-routing metadata  
When MatrixClaw accepts that event  
Then the gateway adapter converts it into a transport-neutral ingress envelope  
And the live runtime consumes only the normalized envelope  
And gateway-specific delivery metadata remains outside the runtime core

## Feature: Matrix session mapping

### Scenario: Matrix room message resumes the mapped persisted session

Given MatrixClaw has a stored mapping from a Matrix room and thread to a runtime session id  
When a new Matrix message arrives for that room and thread  
Then the Matrix gateway reuses the mapped session id  
And the shared live runtime continues the existing conversation  
And the Matrix adapter preserves room and thread routing for the reply path

## Feature: Matrix streamed delivery

### Scenario: Matrix gateway streams assistant progress without changing runtime semantics

Given the live runtime emits streamed assistant events for a gateway-driven run  
When the Matrix gateway projects those events back to the room  
Then the gateway sends incremental assistant output in order  
And the gateway can emit typing or progress updates without changing runtime event ordering  
And the final visible Matrix reply matches the persisted assistant completion

## Feature: Gateway-local delivery state

### Scenario: Delivery retries and dedupe stay outside the runtime

Given a Matrix event may be delivered more than once or a reply send may fail transiently  
When the Matrix gateway processes inbound or outbound traffic  
Then dedupe keys and retry state are stored in the gateway layer  
And the live runtime does not branch on Matrix delivery mechanics  
And duplicate gateway deliveries do not create duplicate runtime turns

## Feature: Optional gateway startup

### Scenario: Matrix gateway remains disabled without explicit configuration

Given MatrixClaw is installed with no Matrix gateway credentials or homeserver settings  
When MatrixClaw starts  
Then the core browser and served transports still start normally  
And the Matrix gateway runner stays disabled  
And configuration clearly reports that the gateway is optional and not active

## Feature: Cross-transport reuse

### Scenario: Browser and Matrix gateway share one persisted session model

Given a conversation was started through the browser transport  
And that conversation has a persisted session id and queued runtime metadata  
When the mapped Matrix room resumes the same session  
Then the shared live runtime continues from the persisted browser state  
And steering and follow-up queue semantics remain correct  
And the Matrix reply reflects the same shared session history visible to browser users
