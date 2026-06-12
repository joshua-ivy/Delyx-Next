# AGENTS.md

## Project Mission

Delyx Next is a local-first, UI-first AI workbench.

It combines:

- Delyx local-first safety, approvals, model routing, source receipts, memory, and diagnostics
- Codex-style project/thread/diff workflow
- Claude-Code-style Explore/Plan/Build/Review agent workflow
- first-class desktop UI as the trust layer

The product promise:

- useful local agent behavior
- visible trust boundaries
- explicit approvals for risky actions
- source-backed answers
- test-backed coding claims
- inspectable run timelines
- diff-first code review
- local-first data by default
- no fake certainty

## Hard Rules

1. Do not blindly copy old Delyx architecture.
2. Use the old Delyx repo as reference, spec, eval source, safety-policy source, and salvage pool.
3. Delyx Next must be UI-first from day one.
4. Do not build a backend-first agent shell with a thin UI.
5. Every runtime concept must have a visible user-facing state.
6. Do not execute file writes, terminal commands, connector actions, durable memory saves, scheduled risky actions, or external agents without approval.
7. Do not claim code was tested unless an execution artifact exists.
8. Do not claim source-backed facts unless EvidenceRecords support them.
9. Do not hide failed, blocked, denied, expired, partial, or uncertain states in the UI.
10. Do not weaken validators or tests to make a feature look complete.
11. Do not add broad dependencies without explaining why.
12. Do not add cloud-first assumptions. Local-first is the default.
13. Do not mark a milestone done without tests, trace markers, or analyzer artifacts.
14. Keep source files focused: aim for 300 lines or fewer, split/review around 400 lines, and treat 500 lines as a hard cap unless the file is generated, declarative config, or has a documented exception.

## UI Rules

- The UI is the product and trust layer.
- Default workflow is Project -> Thread -> Explore -> Plan -> Approve -> Build -> Diff -> Test -> Review.
- Control Center is advanced mode, not the default first experience.
- Diffs are first-class.
- Tests are first-class.
- Approvals are first-class.
- Evidence receipts are first-class.
- Empty, loading, error, blocked, failed, and waiting states must be designed.
- Keyboard navigation must work for primary actions.
- Use shared design-system components and tokens.
- Avoid one-off styling.
- New runtime states require matching UI states.

## Safety Rules

- Risky tools must produce an ActionProposal before execution.
- Approvals must include scope, reason, risk, expiration, and rollback plan where applicable.
- External agents never get broader authority than the current Delyx task.
- File edits must be checkpointed or isolated.
- Terminal commands must be captured as artifacts.
- Secrets must never be stored in the repo.

## Implementation Style

- Prefer small PR-sized changes.
- Prefer small, focused files. Extract components, services, or helpers before a source file grows past the file-size budget.
- Keep Rust domain models typed.
- Keep frontend state truthful and boring.
- Add tests before polish.
- Use deterministic fixtures before live-model testing.
- Record architectural decisions in `SOURCE_OF_TRUTH.md` (§13 ADRs).
- When touching grounded answers, report whether directness, grounding, source quality, or usefulness changed.

## Default Validation Commands

Run relevant checks after every milestone:

```bash
npm run typecheck
npm test
npm run build
cargo test --workspace
```

For Tauri-specific changes:

```bash
npm run tauri dev
```

For eval changes:

```bash
npm run eval:response
npm run eval:agentic
```

## Definition Of Done

A task is not done until:

- code compiles
- tests pass
- feature behavior has a deterministic test or fixture
- user-visible states are truthful
- docs are updated when behavior changes
- risky actions remain approval-gated
- test claims link to execution artifacts
- UI states exist for success, failure, blocked, waiting, empty, and loading states
