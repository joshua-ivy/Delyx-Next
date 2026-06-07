export const cockpitMarkup = String.raw`<div class="delyx deckC" data-screen-label="Command Deck workbench">
  <div class="resize-grip resize-review" title="Resize inspector"></div>
  <div class="resize-grip resize-drawer" title="Resize terminal output"></div>

  <aside class="deck-spine" aria-label="Command Deck spine">
    <button class="deck-spine-logo project-trigger" title="Projects" type="button">D</button>
    <div class="deck-spine-pipe" aria-label="Agent workflow">__SPINE_PIPE__</div>
    <span class="deck-spine-mode mono">__MODE_LABEL__</span>
  </aside>

  <header class="deck-bar">
    <button class="deck-cmd command-trigger" type="button">
      <span class="deck-cmd-k mono">&#8984;K</span>
      <span class="deck-cmd-ph">Type a command, or ask Delyx to steer the run...</span>
    </button>
    __STATUS_PILL__
    <div class="deck-bar-ctx mono">__CONTEXT_CHIPS__</div>
  </header>

  <main class="deck-work">
    <div class="deck-work-scroll">
      <div class="ey">THREAD &middot; __THREAD_ID__ __RUN_LABEL__</div>
      <h1 class="deck-title disp">__THREAD_TITLE__</h1>
      <p class="deck-desc">__THREAD_DESC__</p>

      <div class="deck-conv" aria-label="Thread conversation">__CONVERSATION__</div>

      __THREAD_STATS__

      __BUILD_PROGRESS__

      __WORK_DIFF__

      <button class="deck-termbtn" type="button" aria-expanded="false">
        <span class="deck-pal-key mono">Alt T</span>
        <span class="dot accent"></span>
        __TERMINAL_LABEL__
        <span class="deck-termbtn-x">terminal</span>
      </button>
      <div class="deck-term term mono" data-output-collapsed="false" hidden>
        <div class="deck-term-l muted terminal-history output-block" data-log-line><span class="pr">history &gt;</span> No command history captured.</div>
        <div class="deck-term-l muted output-block" data-log-line>No terminal command has run. Commands require an approval-backed AgentRun artifact.</div>
        <div class="external-agent-stream output-block">
          <div class="deck-term-l muted" data-log-line>No external agent run has been approved or captured.</div>
        </div>
        <div class="deck-term-l output-block" data-log-line><span class="pr">delyx local &gt;</span> <span class="bk"></span></div>
      </div>
    </div>

    <form class="deck-composer deck-comp-form">
      <div class="deck-quicks">
        <button class="deck-quick plan-create" type="button">Create plan</button>
        <button class="deck-quick plan-approve" type="button">Approve it</button>
        <button class="deck-quick plan-question" type="button">Ask question</button>
        <button class="deck-quick plan-review-mode" type="button">Show review</button>
      </div>
      <div class="deck-comp-row">
        <button type="button" class="deck-comp-attach thread-trigger" title="Open threads">+</button>
        <textarea class="deck-comp-input" rows="1" placeholder="Message Delyx with a real local instruction"></textarea>
        <span class="deck-comp-mode pill accent"><span class="dot accent"></span>__COMPOSER_MODE__</span>
        <button class="btn acc deck-comp-send" type="submit">Send <span class="deck-kbd mono">Enter</span></button>
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
