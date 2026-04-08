# Implement

Last updated: 2026-03-28

## Execution Contract
- Do not pause after each checkpoint unless blocked by a real external dependency or a risky ambiguity.
- Treat [Prompt.md](/home/momo/src/zstar/docs/long-horizon/Prompt.md) and [Plans.md](/home/momo/src/zstar/docs/long-horizon/Plans.md) as the durable control plane.
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
2. Treat the Tauri app as the real product boundary, not an optional wrapper.
3. Keep Gateway work outside the runtime core.
4. Keep Node work outside gateway code.
5. Introduce generic boundaries only when they buy separation or testing value.
6. Prefer one real vertical slice over broad placeholder abstractions.

## Current Active Milestone
- Milestone 03 - Tauri Product Boundary and Desktop App Shell

## Iteration Loop
1. Lock the Tauri-first product architecture and packaging assumptions in the long-horizon docs.
2. Create a milestone execution plan under `docs/plans/` for the packaged desktop product slice.
3. Execute the milestone with parallel agent lanes where write scopes are genuinely separable:
   - packaging/runtime integration
   - app-shell/layout architecture
   - onboarding flow
   - product verification
4. Keep runtime, gateway, and future node semantics intact while the product boundary shifts to Tauri.
5. Run focused packaged-product verification, then rerun the broader runtime/gateway checks.
6. Reconcile architectural, packaging, and layout changes back into the long-horizon docs.

Implementation rules for this milestone:
- Do not keep repo-relative UI asset lookup as a product path.
- Do not preserve the preview-style landing shell as the primary desktop experience.
- Keep one single-window app shell for the first serious release.
- Reuse the live runtime and existing policy hooks rather than inventing a second execution loop.
- Preserve current green runtime/gateway verification while shifting the product shell.
- Stage Node work after the Tauri product boundary is stable again.

## Delegation Rules
- Use parallel agents only for genuinely separate write scopes or verification tasks.
- Keep architecture writing, shell composition, and final integration local.
- Use agent-team execution for milestone lanes that can proceed in parallel without write conflicts:
  - Tauri packaging/runtime embedding
  - UI shell and layout decomposition
  - onboarding flow
  - verification harnesses
- Do not delegate the core product boundary decision itself; that is the mainline architectural decision.

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
