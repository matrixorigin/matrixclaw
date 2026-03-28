# Implement

Last updated: 2026-03-28

## Execution Contract
- Do not pause after each checkpoint unless blocked by a real external dependency or a risky ambiguity.
- Treat [Prompt.md](/home/momo/src/matrixclaw/docs/long-horizon/Prompt.md) and [Plans.md](/home/momo/src/matrixclaw/docs/long-horizon/Plans.md) as the durable control plane.
- Update the long-horizon docs whenever milestone status, terminology, or direction changes materially.
- Keep detailed step-by-step implementation plans outside `Plans.md`; use milestone-specific execution artifacts for that.

## Execution Stack
- Top level: long-horizon planner owns roadmap, milestone sequencing, checkpoints, and terminology discipline.
- Milestone level:
  - use existing plan folders under `docs/plans/` when they already match the milestone
  - otherwise create a new milestone-specific execution plan before writing substantial code
- Preferred execution options:
  - `writing-plans` for milestone decomposition
  - `executing-plans` for structured implementation slices
  - targeted subagents only when write scopes or validation tasks can truly run in parallel

## Active Execution Policy
1. Finish architectural vocabulary before adding more capability surface area.
2. Keep Gateway work outside the runtime core.
3. Keep Node work outside gateway code.
4. Introduce generic boundaries only when they buy separation or testing value.
5. Prefer one real vertical slice over broad placeholder abstractions.

## Current Active Milestone
- Milestone 03 - Node Boundary and First Host Capability Slice

## Iteration Loop
1. Write a Node design doc using the new Gateway/Node vocabulary.
2. Decide the first concrete Node slice:
   - screenshot/browser-oriented
   - shell/filesystem-oriented
3. Create a milestone execution plan under `docs/plans/` for that slice.
4. Implement the active Node slice without mixing Gateway and Node concerns.
5. Run focused milestone verification, then rerun the broader gateway/runtime checks.
6. Reconcile any architectural or naming changes back into the long-horizon docs.

Implementation rules for this milestone:
- Do not bury Node semantics inside `execution.rs`, `sandbox_backend.rs`, or gateway modules without first naming the boundary explicitly.
- Keep the first Node slice narrow and testable.
- Reuse the live runtime and existing policy hooks rather than inventing a second execution loop.
- Preserve current green verification while refactoring toward the Node model.

## Delegation Rules
- Use parallel agents only for genuinely separate write scopes or verification tasks.
- Keep architecture writing and final integration local.
- Do not delegate the core boundary design itself; that is the mainline architectural decision.

## Bug Handling
1. Reproduce with the narrowest meaningful test.
2. Fix in the owning layer only:
   - gateway bug -> gateway layer
   - node bug -> node layer
   - runtime bug -> runtime layer
3. Re-run focused verification first, then the broader milestone harness.
4. Record any architectural fallout in `Plans.md`.

## Blocked Protocol
When blocked:
- record the blocker in `Plans.md`
- state the exact missing decision or dependency
- continue with non-blocked documentation, tests, or parallel milestone prep where possible

## Stop Conditions
Stop only when all are true:
- the active milestone is complete
- milestone verification is green
- the long-horizon docs reflect what actually shipped
- the next milestone is selected and staged
