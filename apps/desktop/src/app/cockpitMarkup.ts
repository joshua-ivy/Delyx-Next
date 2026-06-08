export const cockpitMarkup = String.raw`<div class="delyx deckC __EMPTY_CLASS__" data-screen-label="Command Deck workbench">
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
      <div class="deck-desc">__THREAD_DESC__</div>

      <div class="deck-conv" aria-label="Thread conversation">__CONVERSATION__</div>

      __THREAD_STATS__

      __BUILD_PROGRESS__

      __WORK_DIFF__

      __TERMINAL_BLOCK__
    </div>

    <form class="deck-composer deck-comp-form">
      __QUICK_ACTIONS__
      <div class="deck-comp-row">
        <button type="button" class="deck-comp-attach thread-trigger" title="Open threads">+</button>
        <textarea class="deck-comp-input" rows="1" placeholder="Ask Delyx to work locally"></textarea>
        <span class="deck-comp-mode pill accent"><span class="dot accent"></span>__COMPOSER_MODE__</span>
        <button class="btn acc deck-comp-send" type="submit">Send <span class="deck-kbd mono">Enter</span></button>
      </div>
    </form>
  </main>

  <aside class="deck-inspect">
    <div class="deck-ins-head"><span class="ey">__INSPECTOR_LABEL__</span><span class="deck-ins-exp mono">__INSPECTOR_STATUS__</span></div>
    __INSPECTOR__
  </aside>

  __HINTBAR__
</div>`;
