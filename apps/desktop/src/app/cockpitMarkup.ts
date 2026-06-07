export const cockpitMarkup = String.raw`<div class="app" data-screen-label="Cockpit workbench">
  <nav class="rail">
    <div class="logo">D</div>
    <div class="rnav" title="Projects">P</div>
    <div class="rnav on" title="Threads">T</div>
    <div class="rnav" title="Skills">S</div>
    <div class="rnav" title="Memory">M</div>
    <div class="rnav" title="Automations">A</div>
    <div class="sp"></div>
    <div class="rnav" title="Control Center">C</div>
  </nav>

  <header class="top">
    <div class="tcrumb"><span class="nm">delyx-next</span><span class="sub">/ no-thread</span></div>
    <span class="chip"><span class="k">runtime</span><b>not connected</b> local only</span>
    <span class="chip"><span class="k">git</span><b>0</b> uncommitted</span>
    <span class="grow"></span>
    <span class="pill build"><span class="dot"></span>BUILD MODE</span>
    <span class="pill ghost">No active run</span>
    <span class="ticon">K</span>
    <span class="ticon">L</span>
  </header>

  <aside class="side">
    <div class="side-h"><h3>Threads</h3><span class="add">+</span></div>
    <div class="scroll" style="flex:1;">
      <div class="tcard">
        <div class="tt"><span class="md"></span>Empty: no active threads</div>
        <div class="tm"><span class="dt">Now</span><span class="pill ghost">Idle</span></div>
      </div>
    </div>
    <div class="scope">
      <div class="lbl">Approved scope</div>
      <div class="pth">C:/Users/geaux/Downloads/Delyx Next</div>
      <div style="display:flex;gap:6px;"><span class="pill ghost">Local only</span><span class="pill ghost">AGENTS.md</span></div>
    </div>
  </aside>

  <section class="center">
    <div class="scroll">
      <div class="pipe">
        <div class="pstep pending"><div class="pn">01</div><div class="ps">Explore</div><span class="pc">-</span></div>
        <div class="pstep pending"><div class="pn">02</div><div class="ps">Plan</div><span class="pc">-</span></div>
        <div class="pstep pending"><div class="pn">03</div><div class="ps">Build</div><span class="pc">-</span></div>
        <div class="pstep pending"><div class="pn">04</div><div class="ps">Test</div><span class="pc">-</span></div>
        <div class="pstep pending"><div class="pn">05</div><div class="ps">Review</div><span class="pc">-</span></div>
      </div>

      <div class="hero">
        <div class="eye">THREAD &middot; empty <span class="pill ghost" style="font-size:10px;">no AgentRun</span></div>
        <h1>No active thread</h1>
        <p>Create a thread in this project to start real local work. Runtime execution, approvals, diffs, tests, and evidence stay empty until their ledgers exist.</p>
        <div class="stat-row">
          <div class="stat"><div class="sv">0</div><div class="sk">Files touched</div></div>
          <div class="stat"><div class="sv">None</div><div class="sk">Diff</div></div>
          <div class="stat"><div class="sv">Not run</div><div class="sk">Tests</div></div>
          <div class="stat"><div class="sv">0</div><div class="sk">Evidence</div></div>
        </div>
      </div>

      <div class="sec">
        <div class="sec-h"><h4>Plan</h4><span class="pill ghost" style="font-size:10px;">Empty</span><span class="ln"></span><span class="btn plan-create">Create plan</span><span class="btn plan-approve">Approve</span><span class="btn plan-revise">Revise</span><span class="btn plan-cancel">Cancel</span></div>
        <div class="plan-grid">
          <div class="pbox">
            <div class="bh">Files likely to change</div>
            <div class="it"><span class="ix">-</span>No plan has been created.</div>
          </div>
          <div class="pbox">
            <div class="bh">Proposed steps</div>
            <div class="it"><span class="ix">-</span>Create a thread before planning.</div>
          </div>
          <div class="pbox risk">
            <div class="bh">Risks</div>
            <div class="it"><span class="ix">!</span>No risky action has been proposed.</div>
          </div>
          <div class="pbox">
            <div class="bh">Verify and permissions</div>
            <div class="it"><span class="ix">-</span>No test command has been proposed.</div>
            <div class="perm"><span class="pill ghost" style="font-size:10px;">no approvals pending</span></div>
          </div>
        </div>
      </div>

      <div class="sec">
        <div class="sec-h"><h4>Run timeline</h4><span class="ln"></span></div>
        <div class="tl">
          <div class="tnode pending"><div class="tr"><span class="kd">empty</span><span class="ms">No AgentRun events have been recorded for this thread.</span><span class="ts">-</span></div></div>
        </div>
      </div>
    </div>
    <div class="composer">
      <span class="ph">Create a thread before asking Delyx to act...</span>
      <span class="mode-tag">Local</span>
      <span class="send">Send</span>
    </div>
  </section>

  <aside class="review">
    <div class="rev-h"><h3>Review &amp; receipts</h3><span class="pill ghost">0 pending</span></div>
    <div class="rtabs">
      <span class="rtab on">Diff</span>
      <span class="rtab">Tests</span>
      <span class="rtab">Approvals<span class="c">0</span></span>
      <span class="rtab">Evidence</span>
    </div>
    <div class="rev-body">
      <div class="appro">
        <div class="at"><span class="pill ghost">No proposal</span><span style="font-family:var(--mono);font-size:10px;color:var(--fg-3);">none</span></div>
        <h4>No approval requests</h4>
        <div class="kv"><span class="k">Scope</span><span class="v">No file writes, commands, connectors, memory saves, or external agents requested.</span></div>
        <div class="kv"><span class="k">Risk</span><span class="v">No risky action pending.</span></div>
        <div class="kv"><span class="k">Rollback</span><span class="v">No checkpoint exists yet.</span></div>
      </div>

      <div class="dfile">
        <div class="dh"><span class="fn">Diff artifact</span><span class="dst">empty</span></div>
        <div class="dc">
          <div class="dr"><span class="g">-</span><span class="x">No patch or file change has been proposed.</span></div>
        </div>
      </div>

      <div class="dfile test-artifact">
        <div class="dh"><span class="fn">Test artifact</span><span class="dst">not run</span></div>
        <div class="dc">
          <div class="dr"><span class="g">$</span><span class="x">No test command artifact has been captured.</span></div>
        </div>
      </div>

      <div class="sec-h review-findings" style="margin:16px 0 6px;"><h4 style="font-size:12px;">Review &middot; read-only</h4><span class="ln"></span></div>
      <div class="rcpt review-finding"><span class="ri">R</span><div><div class="rn">No review findings</div><div class="rd">Review mode does not edit. Findings appear only after a real ReviewReport is created.</div></div></div>

      <div class="dfile model-settings">
        <div class="dh"><span class="fn">Model routing</span><span class="dst">empty</span></div>
        <div class="dc">
          <div class="dr"><span class="g">-</span><span class="x">No provider settings have been loaded.</span></div>
        </div>
      </div>

      <div class="dfile memory-review">
        <div class="dh"><span class="fn">Memory review</span><span class="dst">empty</span></div>
        <div class="dc">
          <div class="dr"><span class="g">-</span><span class="x">No memory candidates or durable memories exist.</span></div>
        </div>
      </div>

      <div class="dfile skill-review">
        <div class="dh"><span class="fn">Skills</span><span class="dst">inactive</span></div>
        <div class="dc">
          <div class="dr"><span class="g">-</span><span class="x">No skills imported. Third-party skills never auto-activate.</span></div>
        </div>
      </div>

      <div class="dfile automation-review">
        <div class="dh"><span class="fn">Automations</span><span class="dst">paused</span></div>
        <div class="dc">
          <div class="dr"><span class="g">-</span><span class="x">No automation mission contracts. Recurring work starts paused until approved.</span></div>
        </div>
      </div>

      <div class="dfile mobile-review">
        <div class="dh"><span class="fn">Mobile companion</span><span class="dst">not paired</span></div>
        <div class="dc">
          <div class="dr"><span class="g">-</span><span class="x">No mobile companion paired. Mobile cannot access files or terminal by default.</span></div>
        </div>
      </div>

      <div class="dfile release-review">
        <div class="dh"><span class="fn">Release readiness</span><span class="dst">pending</span></div>
        <div class="dc">
          <div class="dr"><span class="g">-</span><span class="x">No release smoke artifact or support bundle export loaded.</span></div>
        </div>
      </div>

      <div class="sec-h" style="margin:16px 0 6px;"><h4 style="font-size:12px;">Evidence &middot; 0 records</h4><span class="ln"></span></div>
      <div class="rcpt"><span class="ri">i</span><div><div class="rn">No evidence records</div><div class="rd">Claims stay unsupported until a real EvidenceRecord is created.</div></div></div>
    </div>
  </aside>

  <footer class="drawer">
    <div class="dr-h">
      <div class="dr-tabs"><span class="on">Terminal</span><span>Logs</span><span>External agent</span></div>
      <span class="pill ghost">Idle</span>
    </div>
    <div class="term">
      <div class="dm">No terminal command has run. Commands require an approval-backed AgentRun artifact.</div>
      <div class="external-agent-stream">
        <div class="dm">No external agent run has been approved or captured.</div>
      </div>
      <div><span class="pr">delyx local &gt;</span> <span class="bk"></span></div>
    </div>
  </footer>
</div>`;
