# Migration From Old Delyx

Old Delyx is the baseline reference, spec source, eval source, safety-policy source, and salvage pool.

Reference:

- https://github.com/joshua-ivy/Delyx

Delyx Next is a clean rebuild. Do not copy old architecture blindly.

## Preserve

Preserve these strengths:

- Windows-first local desktop app
- Tauri + React + TypeScript + Rust + SQLite unless a decision record proves otherwise
- visible approval boundaries
- local workspace safety
- model/runtime routing
- coding lane
- sandbox-backed execution evidence
- source-backed research
- memory with approval
- diagnostics/control center
- task/run timeline concepts
- skills as visible capabilities
- connector boundaries
- mobile companion later
- automations later
- packaging/release lessons

## Rebuild Differently

The old app grew into a broad cockpit. Delyx Next should start with the narrow workflow:

```text
Project -> Thread -> Explore -> Plan -> Approve -> Build -> Diff -> Test -> Review
```

Use the old repo to answer:

- which user promises mattered?
- which safety policies worked?
- which data models are worth adapting?
- which tests/evals caught real regressions?
- which UX states users needed?
- which provider integrations are worth keeping?

Do not use it as proof that old module boundaries are correct.

## Known Old Patterns To Avoid

Avoid:

- cockpit complexity before the basic workbench loop works
- phrase-list-only routing as the core reasoning system
- final-answer phrase validators as evidence
- symbol-name overlap treated as proof
- hidden failed states
- hidden blocked states
- tested-code claims without artifacts
- broad always-allow policies too early
- backend concepts with no UI surface
- UI screens that look complete while runtime truth is missing

## Salvage Candidates

Likely salvage areas:

- provider configuration concepts
- Ollama/OpenAI-compatible provider support
- local app data conventions
- approval policy language
- diagnostics language
- source receipt model ideas
- sandbox evidence behavior
- release smoke scripts
- mobile companion lessons
- connector boundary ideas

Every salvage should pass this filter:

```text
Does this help the Project -> Thread -> Plan -> Diff -> Test -> Review loop?
Does it preserve or improve local-first safety?
Can it be represented clearly in the UI?
Does it have tests or can we add deterministic tests?
```

## Migration Docs Needed Later

When real code migration begins, document:

- old source path
- new destination path
- adapted behavior
- behavior intentionally dropped
- safety implications
- test coverage
- UI state exposed

## App Identity

Use a separate app identity for Delyx Next so old Delyx and Delyx Next can coexist during development.

Recommended app ID:

```text
com.geaux.delyxnext
```

Record any change to this in `docs/ARCHITECTURE.md`.

