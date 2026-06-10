# Delyx Next — Campaign Mode (War RPG Engine)

> **Status (2026-06-09): C1–C8 implemented.** All seven era packs ship; the turn loop runs
> dice → layered GM prompt → streamed narration → delta apply → async QA/QC; the Campaign
> UI is the dice icon in the rail (Ctrl+G). Remaining polish lives in §10.

Local-first roleplay engine that beats Character.AI by separating the **narrator** (local model)
from the **state** (SQLite). The model never owns the truth — the app does.

Audience: single local user (family use). No cloud, no accounts, no content leaves the machine.
Built for long-running campaigns played by a kid, so a **parent-controlled content dial** is a
first-class feature, not an afterthought.

---

## 1. Why this beats Character.AI

| Character.AI weakness | Campaign Mode answer |
|---|---|
| Forgets events after ~40 messages | Campaign state lives in SQLite; injected into every prompt. Wounds, deaths, inventory never "un-happen". |
| No game state — pure improv | Character sheet, squad roster, timeline position, morale, inventory are structured rows, not prose. |
| One AI = one character | The model is a **Game Master**: narrates the world and voices every NPC in one streamed reply. |
| AI lets the player always win | The **app** rolls dice and tells the model the outcome. The model narrates a result it was given. |
| Hallucinates history | **Lore packs** (curated era files) ride the existing attachment/context-pack pipeline into the prompt. |
| No fact-checking | Async **Continuity QA/QC** pass via the existing `cli_review`-style CLI bridge (Claude/Codex CLI, subscription auth, `--safe-mode`). Never blocks the reply — append-only follow-up, same rule as code QA/QC. |
| Cloud filter kills war stories | Local model + per-campaign content rating set by the parent, enforced in the GM system prompt. |

---

## 2. Core concepts

```
Era Pack (static content)        Campaign (one playthrough)         Turn (one exchange)
─────────────────────────       ───────────────────────────        ─────────────────────
era timeline + events            player character sheet             player input
weapons / units / slang          squad roster (alive/dead)          app resolves checks (dice)
historical figures               world clock (in-era date)          GM prompt assembled
scenario seeds                   location / front position          stream narration
GM style + rating overlays       event log + memory summary         state-delta applied
                                 content rating (parent-set)        async continuity QA/QC
```

- **Era Pack** — shippable content folder (Star Wars, Revolutionary War, Civil War, WW1, WW2,
  Korea, Vietnam…). Adding a war = authoring a pack, zero engine code.
- **Campaign** — one playthrough of one pack: character + persistent world state. Long-lived,
  resumable across app restarts.
- **Turn** — the heartbeat: player text in → app-side resolution → GM narration out → state delta.

---

## 3. Data model (new migration `0002_campaign_mode.sql`)

Follows the existing rusqlite/bridge pattern (`task_threads`, `thread_messages` style:
TEXT ids, JSON columns for nested structs, ISO timestamps).

```sql
CREATE TABLE campaigns (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL,            -- campaigns live under a project like threads do
  era_pack_id TEXT NOT NULL,           -- "ww1", "civil-war", "star-wars", ...
  title TEXT NOT NULL,                 -- "Harlem Hellfighters, 1918"
  status TEXT NOT NULL,                -- active | completed | abandoned
  content_rating TEXT NOT NULL,        -- story | heroic | historical   (see §7)
  world_date TEXT NOT NULL,            -- in-era clock, e.g. "1918-03-21"
  location TEXT NOT NULL,              -- current scene anchor, e.g. "Forward trench, St. Mihiel"
  scenario_id TEXT,                    -- which pack scenario seeded this campaign
  memory_summary TEXT NOT NULL DEFAULT '',  -- rolling GM-visible recap (see §6.3)
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE campaign_characters (
  id TEXT PRIMARY KEY,
  campaign_id TEXT NOT NULL REFERENCES campaigns(id) ON DELETE CASCADE,
  kind TEXT NOT NULL,                  -- player | npc
  name TEXT NOT NULL,
  role TEXT NOT NULL,                  -- "rifleman", "medic", "droid", "war correspondent"
  status TEXT NOT NULL,                -- active | wounded | missing | dead | departed
  sheet_json TEXT NOT NULL,            -- stats, skills, traits (pack-defined schema)
  inventory_json TEXT NOT NULL,
  bonds_json TEXT NOT NULL,            -- relationships: trust/grudges toward other characters
  notes TEXT NOT NULL DEFAULT '',      -- GM-authored persistent facts ("limps since Belleau Wood")
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE campaign_turns (
  campaign_id TEXT NOT NULL REFERENCES campaigns(id) ON DELETE CASCADE,
  turn_index INTEGER NOT NULL,
  player_text TEXT NOT NULL,
  resolution_json TEXT NOT NULL,       -- dice rolls, check results the app produced (see §5)
  narration TEXT NOT NULL,             -- GM output shown to the player
  state_delta_json TEXT NOT NULL,      -- the structured delta the GM proposed + app applied
  qaqc_status TEXT NOT NULL DEFAULT 'pending',  -- pending | clean | corrected | skipped
  qaqc_notes TEXT,                     -- continuity findings (appended async, never blocking)
  created_at TEXT NOT NULL,
  PRIMARY KEY (campaign_id, turn_index)
);

CREATE TABLE campaign_events (
  id TEXT PRIMARY KEY,
  campaign_id TEXT NOT NULL REFERENCES campaigns(id) ON DELETE CASCADE,
  turn_index INTEGER NOT NULL,
  kind TEXT NOT NULL,                  -- death | wound | promotion | item | bond | location | historical
  summary TEXT NOT NULL,               -- one line: "Pvt. Mills killed by sniper at Vaux"
  created_at TEXT NOT NULL
);
```

Design notes:

- `campaign_turns` is the RPG analogue of `thread_messages` — but each row carries the
  **resolution** and **state delta** alongside the prose, so replay/debug/QA-QC all have
  structured ground truth.
- `campaign_events` is the **canon ledger**: a compact, queryable list of facts that must never
  be contradicted. It is small enough to inject into every prompt even in 200-turn campaigns
  (the full transcript is not — see §6.3).
- Characters are rows, not prose, so "who is alive" is a `WHERE status != 'dead'` query, not a
  model memory test.

---

## 4. Era Packs

A pack is a folder under `packs/<era-id>/` (shipped in-repo, user-extensible later):

```
packs/ww1/
  pack.json            -- manifest
  lore/                -- markdown files, chunked by the EXISTING attachment parser
    timeline.md        -- dated real events the world clock walks through
    weapons.md         -- equipment with era-correct names/behaviors ("the Chauchat jams")
    daily-life.md      -- trench routine, slang, food, letters home
    figures.md         -- real people the player may encounter
  scenarios.json       -- starting situations (see below)
  sheet-schema.json    -- what a character sheet looks like in this era
```

`pack.json` manifest:

```json
{
  "id": "ww1",
  "title": "The Great War (1914–1918)",
  "gmStyle": "Second person, present tense. Grounded, sensory, human-scale. NPCs have names and wants.",
  "checks": ["grit", "wits", "aim", "charm"],
  "ratingOverlays": {
    "story":      "Danger is real but violence is described like a classic adventure novel...",
    "heroic":     "Combat has stakes and loss, described like a PG-13 war film...",
    "historical": "Unflinching but never gratuitous; honor the reality of the era..."
  }
}
```

`scenarios.json` — each entry seeds a campaign:

```json
{
  "id": "doughboy-1918",
  "title": "Over There — AEF Rifleman, Spring 1918",
  "startDate": "1918-03-15",
  "startLocation": "Training camp near Chaumont, France",
  "opening": "...", 
  "squad": [ { "name": "Sgt. Calloway", "role": "squad leader", "trait": "by-the-book" }, ... ],
  "timelinePressure": ["1918-03-21 German Spring Offensive begins", "1918-05-28 Cantigny", ...]
}
```

**Reuse, not rebuild:** lore markdown is parsed/chunked by the existing attachment pipeline
(`parse_attachment` → `attachment_chunks`) and selected per-turn through a **context pack** with
a token budget — exactly the machinery PR1–PR3 of the Projects/Attachments plan built. A pack is
registered as a set of pre-approved local attachments on the campaign's project, so lore
injection is relevance-ranked chunks, not "stuff the whole pack in the prompt."

Launch packs: **WW1 first** (best stress test of memory/consequence/tone), then Civil War, WW2,
Revolutionary War, Star Wars, Korea, Vietnam. Engine identical for all.

---

## 5. The turn loop (the engine)

New Rust module `campaign_bridge.rs`, Tauri commands following house style:

| Command | Purpose |
|---|---|
| `campaign_pack_list()` | Enumerate installed packs + scenarios |
| `campaign_create(req)` | New campaign from pack + scenario + rating + character choices |
| `campaign_snapshot(project_id)` | Hydrate campaigns, characters, recent turns, events (mirrors `thread_run_snapshot`) |
| `campaign_turn_prompt(req)` | Builds the layered GM message list from canon (§6); frontend streams it through the existing `model_chat_stream` / `model-stream` path (cancel = `model_chat_cancel`) |
| `campaign_turn_commit(req)` | Persists the finished turn (player text + narration); resolution/delta attach here in C3/C4 |
| `campaign_turns(campaign_id)` | Hydrates the turn timeline for the play view |
| `campaign_update_settings(req)` | Rating change, rename, abandon (rating change is parent-gated, §7) |

`campaign_turn_submit` pipeline:

```
1. RESOLVE (app, deterministic — Rust, no model)
   Classify whether the player's action is risky (keyword + verb heuristics per pack `checks`,
   or an explicit "Try it" UI affordance). If so: roll 2d6 + stat against difficulty.
   Output: resolution_json e.g. {"check":"aim","roll":[4,2],"stat":1,"total":7,"outcome":"partial"}
   Rolls are SHOWN to the player (dice are fun and visible fairness is the anti-Character.AI move).

2. ASSEMBLE PROMPT (see §6)

3. NARRATE (existing model pipeline)
   `model_chat_stream` on delyx-local / ollama-local — same streaming + cancel path the
   chat UI already uses. Tokens stream into the Campaign view immediately.

4. EXTRACT STATE DELTA
   The GM prompt ends with: "After the scene, output a fenced ```delta json block."
   Rust parses the trailing fenced block (same trailing-block extraction trick as
   `split_review_text` in cli_review.rs), strips it from the narration shown to the player.
   Delta schema:
     { "events": [{"kind":"wound","summary":"..."}],
       "characters": [{"name":"Sgt. Calloway","status":"wounded","notes":"shrapnel, left arm"}],
       "inventory": {"add":["German pistol"],"remove":["last ration tin"]},
       "clock": {"advance":"6h"}, "location": "Shell crater, no-man's-land" }
   APP VALIDATES the delta (unknown character → reject that entry; dead characters can't act;
   clock only moves forward). Apply to SQLite. A malformed/missing delta is non-fatal:
   narration still posts, delta retried by the QA/QC pass.

5. PERSIST turn row + events; bump campaign updated_at.

6. QA/QC (async, append-only — never gates the narration; same rule as code review QA/QC)
   Spawn the CLI reviewer (claude -p --safe-mode / codex exec read-only, cheap model) with:
   canon event ledger + character roster + pack timeline + this turn's narration.
   Asks: continuity breaks? anachronisms? rating violations? missed delta facts?
   Result lands as qaqc_notes on the turn; "corrected" findings (e.g. "M1 Garand didn't exist
   in 1917") write a campaign_event of kind=historical correction and a GM-visible note so the
   NEXT turn quietly fixes canon. UI shows a small ✓/⚠ chip on the turn — kid can ignore it,
   parent can read it.
```

Failure handling matches the existing engine-crash recovery posture: if the local model dies
mid-turn, the turn row is not written, the player sees a retry affordance, and the campaign
state is untouched (turns are atomic).

---

## 6. Prompt assembly (the GM brain)

Built per-turn in Rust. Target budget ~6–8k tokens on a 30B/32k context.

### 6.1 Layered system prompt

```
[1] GM CONTRACT (engine, fixed)
    You are the Game Master. You narrate the world and voice every NPC. You never speak for
    the player or decide their actions. You never roll dice or decide success — the RESOLUTION
    block tells you what happened; your job is to narrate it with consequences. Keep scenes
    2–4 paragraphs, end on a hook or a question. End with the ```delta block.

[2] ERA VOICE (pack: gmStyle)
[3] RATING OVERLAY (pack: ratingOverlays[campaign.content_rating])   ← parent dial, §7
[4] CANON — WORLD: world_date, location, upcoming timelinePressure entries within ~30 days
[5] CANON — CHARACTERS: roster rows (name, role, status, key notes, bonds) — dead means dead
[6] CANON — EVENT LEDGER: campaign_events summaries (compact, the whole list)
[7] MEMORY SUMMARY: rolling recap (§6.3)
[8] LORE CHUNKS: top-k relevant attachment chunks for the current scene/action (context pack)
[9] QA/QC NOTES: any pending continuity corrections to weave in quietly
```

### 6.2 Message window

Last ~10 turns verbatim as alternating user/assistant messages (player_text / narration), then
the current turn: player text + `RESOLUTION: {...}` appended by the app.

### 6.3 Rolling memory summary

Every N turns (N=15) or when the verbatim window would overflow budget, a background summarize
call (same local model, non-streaming `model_chat`) compresses the oldest turns into
`campaigns.memory_summary` ("Acts so far…"). The event ledger keeps hard facts; the summary
keeps narrative texture. This is the piece that makes turn 200 remember turn 3 — the thing
Character.AI structurally cannot do.

---

## 7. Content rating (the parent dial)

Per-campaign `content_rating`, chosen at creation, enforced as prompt overlay [3] **and**
checked by the QA/QC pass (defense in depth — local models drift):

- **story** (~PG) — peril and stakes, adventure-novel violence, no gore, fade-to-black on death
  details, no period slurs, themes of courage/duty/friendship.
- **heroic** (~PG-13) — real loss, war-film intensity, injuries acknowledged not lingered on.
- **historical** — honest to the era (gas, shell shock, casualty scale), never gratuitous.

Changing a campaign's rating requires the app-level parent confirmation (reuse the existing
approval-card pattern — it's exactly an approval gate). Default for new campaigns: **story**.

This is itself a differentiator: Character.AI gives you one global corporate filter; Delyx
gives the parent a per-campaign dial with an auditable QA/QC trail.

---

## 8. UI integration

Campaign Mode is a **third top-level view** in the existing router, not a retrofit of threads
(threads carry agent-run/plan/approval semantics that don't apply):

- `FocusShell.tsx`: `view: "home" | "thread" | "settings" | "campaign"`. Entry points: a
  "Campaigns" card/section on `FocusHome`, command palette entry, `Ctrl+G`.
- **`CampaignHome.tsx`** — pack picker (era cards), scenario picker, character creation
  (name + 2 trait picks from `sheet-schema.json`), rating selector; list of resumable
  campaigns with "last played" + current world date.
- **`CampaignThread.tsx`** — the play view:
  - Narrative timeline (reuses `focusMarkdown` rendering; narration is markdown).
  - Streaming via the existing `model-stream` listener pattern in `modelClient.ts`.
  - **Dice moments rendered as inline artifact blocks** (the peeker pattern from
    FocusThread): roll animation, check name, outcome.
  - Right rail (collapsible): character sheet, squad roster with status pips, world
    date/location, recent events ledger.
  - Composer with quick-action chips the pack defines ("Look around", "Talk to…", "Take
    cover", "Press on") — kid-friendly, keyboard optional.
  - Turn QA/QC chip (✓/⚠) once the async pass lands — taps open the findings.
- **`campaignClient.ts`** under `apps/desktop/src/features/campaigns/` — same typed
  invoke/listen client pattern as `modelClient.ts`.

Model selection reuses the existing model menu; campaigns default to the **Coding-route local
model** (delyx-local or ollama-local). CLI adapters appear only as the QA/QC reviewer, never
the narrator (latency + the CLI-over-API rule).

---

## 9. Build plan (PR sequence)

Same discipline as the Projects/Attachments 12-PR tracker — each PR lands green and playable-ish:

| PR | Scope | Proves |
|---|---|---|
| **C1** | Migration `0002_campaign_mode.sql` + `campaign.rs` records + `campaign_bridge.rs` CRUD (`campaign_create/snapshot/pack_list`) + Rust tests | Persistence layer |
| **C2** | Turn loop v1: prompt assembly + `campaign_turn_submit` streaming narration, **no dice, no delta** (pure GM chat with canon injection) + WW1 pack stub (manifest + 1 scenario + timeline.md) | The GM voice works on the 30B |
| **C3** | State deltas: trailing ```delta extraction, validation, apply; event ledger; character status updates | The model can't forget |
| **C4** | Dice resolution: app-side checks, RESOLUTION injection, inline dice UI block | Consequences are real |
| **C5** | Campaign UI proper: CampaignHome + CampaignThread + right rail + quick-action chips | Playable by a kid |
| **C6** | Memory: rolling summary compaction + lore chunks via context packs (wire pack lore through attachment parsing) | 100+ turn campaigns |
| **C7** | QA/QC continuity pass (async CLI reviewer) + rating overlays + parent gate on rating change | "Knows the war" + safe |
| **C8+** | Content sprints: Civil War, WW2, Revolutionary War, Star Wars, Korea, Vietnam packs (authoring only) | The library |

C2 is the de-risking milestone: if the local 30B can't hold GM voice + delta discipline, we
adjust (smaller delta schema, two-pass narrate-then-extract) before building UI on top.

---

## 10. Open questions (decide during C1/C2)

1. **Delta reliability on 30B** — single-pass trailing block vs. a second cheap extraction call.
   Start single-pass; the QA/QC pass already backstops missed deltas.
2. **Pack distribution** — in-repo `packs/` now; later a user folder (`%APPDATA%/Delyx/packs`)
   so packs can be added without rebuilding.
3. **Multiplayer-ish** — two characters (siblings) in one campaign is just two `player` rows
   and a "whose turn" marker; cheap to add post-C5 if wanted.
4. **Star Wars IP** — local personal use only; pack never ships outside this machine's builds.
