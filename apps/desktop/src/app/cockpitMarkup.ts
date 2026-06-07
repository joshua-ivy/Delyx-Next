export const cockpitMarkup = String.raw`<div class="delyx deckC" data-screen-label="Command Deck workbench">
  <div class="resize-grip resize-review" title="Resize inspector"></div>
  <div class="resize-grip resize-drawer" title="Resize terminal output"></div>

  <aside class="deck-spine" aria-label="Command Deck spine">
    <button class="deck-spine-logo project-trigger" title="Projects" type="button">D</button>
    <div class="deck-spine-pipe" aria-label="Agent workflow">__SPINE_PIPE__</div>
    <button class="deck-spine-action thread-trigger" title="Threads" type="button">+</button>
    <span class="deck-spine-mode mono">__MODE_LABEL__</span>
  </aside>

  <header class="deck-bar">
    <button class="deck-cmd command-trigger" type="button">
      <span class="deck-cmd-k mono">Ctrl K</span>
      <span class="deck-cmd-ph">Type a command, open a manager, or steer the active run</span>
    </button>
    __STATUS_PILL__
    <div class="deck-bar-ctx mono">__CONTEXT_CHIPS__</div>
    <button class="deck-icon-btn theme-trigger" title="Switch to light theme" type="button">L</button>
  </header>

  <main class="deck-work">
    <div class="deck-work-scroll">
      <div class="ey">THREAD &middot; __THREAD_ID__ __RUN_LABEL__</div>
      <h1 class="deck-title disp">__THREAD_TITLE__</h1>
      <p class="deck-desc">__THREAD_DESC__</p>

      <div class="deck-conv" aria-label="Thread conversation">__CONVERSATION__</div>

      __THREAD_STATS__

      <section class="deck-section">
        <div class="sec-h"><h4>Plan</h4><span class="pill ghost micro">__PLAN_STATE__</span><span class="ln"></span><span class="btn plan-create">Create plan</span><span class="btn plan-approve">Approve</span><span class="btn plan-edit">Edit plan</span><span class="btn plan-question">Ask question</span><span class="btn plan-review-mode">Read-only review</span><span class="btn plan-revise">Revise</span><span class="btn plan-cancel">Cancel</span></div>
        __PLAN_GRID__
      </section>

      <section class="deck-section">
        <div class="sec-h"><h4>Run timeline</h4><span class="ln"></span></div>
        <div class="tl">__TIMELINE__</div>
      </section>

      <section aria-label="Terminal, logs, and external agent drawer" class="deck-section deck-terminal-panel">
        <div class="deck-term-head">
          <div class="dr-tabs"><span class="on">Terminal</span><span>Logs</span><span>External agent</span></div>
          <label class="log-search-wrap"><span>Search logs</span><input class="log-search" aria-label="Search drawer logs" type="search" placeholder="Search logs" /></label>
          <button class="output-collapse" type="button" aria-pressed="false">Collapse output</button>
          <button class="terminal-action terminal-copy" type="button">Copy output</button>
          <button aria-disabled="true" class="terminal-action terminal-jump-error" disabled title="Requires a captured error artifact" type="button">Jump to error</button>
          <button aria-disabled="true" class="terminal-action terminal-open-file" disabled title="Requires a referenced file artifact" type="button">Open file</button>
          <button aria-disabled="true" class="terminal-action terminal-rerun" disabled title="Requires a captured command artifact" type="button">Rerun</button>
          <button aria-disabled="true" class="terminal-action terminal-approve-rerun" disabled title="Requires a pending rerun approval" type="button">Approve rerun</button>
        </div>
        <div class="term mono" data-output-collapsed="false">
          <div class="dm terminal-history output-block" data-log-line><span class="pr">history &gt;</span> No command history captured.</div>
          <div class="dm output-block" data-log-line>No terminal command has run. Commands require an approval-backed AgentRun artifact.</div>
          <div class="external-agent-stream output-block">
            <div class="dm" data-log-line>No external agent run has been approved or captured.</div>
          </div>
          <div class="output-block" data-log-line><span class="pr">delyx local &gt;</span> <span class="bk"></span></div>
        </div>
      </section>
    </div>

    <form class="deck-composer deck-comp-form">
      <div class="deck-quicks">
        <button class="deck-quick plan-create" type="button">Create plan</button>
        <button class="deck-quick plan-question" type="button">Ask question</button>
        <button class="deck-quick plan-review-mode" type="button">Read-only review</button>
      </div>
      <div class="deck-comp-row">
        <textarea class="deck-comp-input" rows="1" placeholder="Message Delyx with a real local instruction"></textarea>
        <span class="deck-comp-mode pill accent"><span class="dot"></span>Local</span>
        <button class="btn acc deck-comp-send" type="submit">Send</button>
      </div>
    </form>
  </main>

  <aside class="deck-inspect">
    <div class="deck-ins-head"><span class="ey">Needs you now</span><span class="deck-ins-exp mono">__INSPECTOR_STATUS__</span></div>
    __INSPECTOR__
  </aside>

  <div class="deck-hintbar mono">
    <span><b>Ctrl K</b> commands</span>
    <span><b>Enter</b> send</span>
    <span><b>Alt T</b> terminal</span>
    <span><b>Esc</b> close palette</span>
  </div>
</div>`;
