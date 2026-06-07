# Delyx Next UI Principles

The UI is the product and the trust layer.

Delyx Next should feel like a serious desktop workbench from the first app-shell PR, while staying honest about which runtime paths are still skeletal.

Target feel:

```text
VS Code + Linear + Codex-style workbench + local control panel
```

## Design Goals

The interface should be:

- professional
- calm
- fast
- dense but readable
- inspectable
- truthful about failure
- useful with real runtime data or honest empty states before runtime depth exists

It should not feel:

- toy-like
- gamer-like
- like a generic chat clone
- like a backend demo with a thin window
- like a dashboard that hides the main workflow

## Default Layout

The primary app shell has four persistent regions:

```text
Top bar:
  Project, branch/worktree, model, mode, run status

Left sidebar:
  Projects, threads, skills, automations later, memory, settings

Center panel:
  Active task thread, chat/composer, plan, step timeline

Right panel:
  Review panel with diff, tests, approvals, evidence, findings

Bottom drawer:
  Terminal, logs, test output, external agent transcript
```

Advanced mode adds:

- Control Center
- Run Inspector
- model routing
- tool policy
- memory manager
- automation contracts
- diagnostics

## First Screens To Build

PR 2 established the real-feeling shell prototype. Current shipped UI must use real runtime data or honest empty, loading, error, waiting, blocked, failed, denied, expired, partial, and success states.

Screens and panels:

- project/thread sidebar
- active task thread
- plan panel
- approval drawer
- diff review panel
- test panel
- evidence panel
- terminal/log drawer
- external agent transcript panel
- command palette with safe local shell actions
- settings/model status placeholder
- blocked state
- failed state
- empty state
- loading state
- error state

## UI State Requirements

No major runtime state should be invisible.

Required task states:

- idle
- exploring
- planning
- waiting_for_approval
- building
- testing
- reviewing
- blocked
- failed
- done

Required UI support states:

- empty
- loading
- error
- denied
- expired
- partial
- untested
- insufficient evidence

## Design System

PR 2 should create a small design system with stable tokens and reusable components.

Recommended stack:

- React
- TypeScript
- Vite
- Tauri v2
- CSS variables for tokens
- Radix UI primitives where useful
- Lucide icons
- TanStack Query once real server state exists
- Zustand only if local UI state grows beyond simple React state
- Monaco or CodeMirror later for richer code/diff surfaces
- xterm.js or similar later for terminal emulation

Tailwind is not required for the initial implementation. Prefer CSS variables and reusable components unless a decision record changes this.

## Tokens

Create tokens for:

- background
- surface
- border
- muted text
- primary text
- accent
- success
- warning
- danger
- info
- focus ring
- spacing
- radius
- shadow
- typography

Status colors must be consistent across the whole app.

## Components

PR 2 should include reusable components for:

- Button
- IconButton
- Badge
- StatusPill
- RiskBadge
- Dialog
- Drawer
- Tabs
- Tooltip
- SplitPane
- EmptyState
- ErrorState
- LoadingState
- CommandPalette
- LogViewer

Feature components should use these primitives rather than one-off styles.

## Component Size

Keep UI files focused:

- target component file size: 300 lines or fewer
- split/review threshold: 400 lines
- hard cap: 500 lines unless generated, declarative config, or explicitly documented

When a feature grows, split state, view-model mapping, panel chrome, and repeated row/card components into separate files.

## Interaction Rules

Use familiar controls:

- icons inside toolbar buttons
- tabs for panel switching
- drawers for approvals and logs
- segmented controls for modes
- toggles for binary settings
- menus for option sets
- tooltips for unfamiliar icons
- keyboard navigation for primary shell actions

Do not use visible text to explain the app's own controls when the control can be made clear through structure, labels, and tooltips.

## Accessibility

The workbench must support:

- visible focus states
- keyboard navigation for major shell actions
- accessible dialogs and drawers
- readable contrast in light and dark themes
- non-color status labels
- scrollable long logs
- collapsible long output
- stable layout dimensions for panels and toolbars

## Prototype Data Rule

Mock data was a product design tool for the first shell prototype. It must not ship as runtime state.

Historical PR 2 prototype states remain coverage targets for fixtures and tests:

- a real-looking project
- multiple task threads
- active plan
- pending approval
- approved approval
- denied or expired approval
- changed files
- unified diff
- passing test output
- failing test output
- evidence receipts
- blocked run
- failed run
- done run
- external agent transcript
- untested state
- insufficient evidence state

Fixture click-through:

```text
Project
-> Thread
-> Plan
-> Approval
-> Diff
-> Test
-> Evidence
-> Blocked
-> Failed
-> Done
```

## UI-First Acceptance Tests

The UI is first-class when tests prove:

- the shell renders with real runtime data, honest empty states, or deterministic test fixtures
- major panels are reachable by click and keyboard
- task states render with correct status labels
- blocked and failed states are visible
- pending, denied, and expired approvals are visible
- test panel can render tested and untested states
- evidence panel can render supported and insufficient-evidence states
- diff panel shows changed files and patch content
- major screens have empty, loading, and error states
