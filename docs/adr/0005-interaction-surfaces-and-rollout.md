# Interaction surfaces and rollout

- Status: Accepted
- Date: 2026-09-19

## Context

The product concept includes several complementary ways to route application audio.
The selected visual references must remain recognizable across native platform
implementations, while fragile operating-system integrations must not delay the
reliable routing core.

## Decision

Use one shared visual language, information hierarchy, color system, and component
specification across these interaction surfaces:

1. A compact tray or menu-bar popup is the primary, fastest interface.
2. An expanded mini mixer provides per-application output and volume controls.
3. A taskbar-adjacent application selector is added after the core is stable.
4. System volume-panel integration is an optional platform module added after the
   core is stable.

The primary popup corresponds to selected layout 1, the taskbar selector to layout
2, the mini mixer to layout 5, and the volume integration to layout 6 from the
original product exploration.

Any integration with another application's taskbar menu or the Windows shell is
isolated from the routing core. It must use supported operating-system mechanisms
where possible and must fail without affecting normal routing.

## Consequences

- The first production UI implements the compact popup and mini mixer.
- Later platform integrations reuse the same domain actions and visual tokens.
- Native implementations may follow platform conventions without changing the
  information hierarchy or product identity.
- Explorer or shell integration cannot become a dependency of core routing.
