import { CircleAlert, Terminal } from "lucide-react";

import { Badge } from "../../design-system/Badge";
import { StateBlock } from "../../design-system/StateBlock";
import type { TestRunViewModel } from "../../app/types";

export function TestPanel({ tests }: { tests: TestRunViewModel[] }) {
  return (
    <div className="test-panel" data-testid="test-panel">
      {tests.map((test) => (
        <article className="test-card" key={test.id}>
          <header>
            <Terminal size={16} />
            <strong>{test.command}</strong>
            <Badge tone={test.status === "passed" ? "success" : test.status === "failed" ? "danger" : "warning"}>
              {test.status.replace("_", " ")}
            </Badge>
          </header>
          <dl>
            <div>
              <dt>Working directory</dt>
              <dd>{test.cwd}</dd>
            </div>
            <div>
              <dt>Exit code</dt>
              <dd>{test.exitCode ?? "not run"}</dd>
            </div>
            <div>
              <dt>Duration</dt>
              <dd>{test.durationMs}ms</dd>
            </div>
          </dl>
          <pre>
            {test.output.map((line) => (
              <code key={line}>{line}</code>
            ))}
          </pre>
          {test.status === "not_run" && (
            <StateBlock
              detail="No approved test command was executed."
              title="Not tested"
              tone="warning"
              action={<CircleAlert size={16} />}
            />
          )}
        </article>
      ))}
    </div>
  );
}
