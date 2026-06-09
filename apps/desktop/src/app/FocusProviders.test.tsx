import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (cmd: string, args?: unknown) => invoke(cmd, args) }));

import { FocusProviders } from "./FocusProviders";

afterEach(() => {
  cleanup();
  invoke.mockReset();
});

function status(anthropic: boolean, openai: boolean) {
  return {
    providers: [
      { id: "anthropic", label: "Anthropic", hasKey: anthropic },
      { id: "openai", label: "OpenAI", hasKey: openai },
    ],
  };
}

describe("FocusProviders", () => {
  it("detects CLIs, shows key state, and saves a pasted key through the bridge", async () => {
    invoke.mockImplementation((cmd: string) => {
      if (cmd === "secret_status") return Promise.resolve(status(false, false));
      if (cmd === "external_agent_status") {
        return Promise.resolve({
          adapters: [
            { id: "claude-code", kind: "claude_code", label: "Claude Code", status: "available", detail: "found" },
            { id: "codex-cli", kind: "codex_cli", label: "Codex CLI", status: "missing", detail: "missing" },
          ],
        });
      }
      if (cmd === "secret_set") return Promise.resolve(status(true, false));
      return Promise.resolve(status(false, false));
    });

    render(<FocusProviders />);

    // CLI detection: Claude detected, Codex shows the install command.
    expect(await screen.findByText("detected")).toBeTruthy();
    expect(screen.getByText(/npm i -g @openai\/codex/)).toBeTruthy();

    // Anthropic key starts unset, then saving flips it to "set".
    const input = await screen.findByLabelText("Anthropic API key");
    fireEvent.change(input, { target: { value: "sk-test-key" } });
    fireEvent.click(screen.getAllByText("Save")[0]);

    await waitFor(() => expect(invoke).toHaveBeenCalledWith("secret_set", { providerId: "anthropic", value: "sk-test-key" }));
    expect(await screen.findByText("set")).toBeTruthy();
  });

  it("falls back to a desktop-only notice when the bridge is unavailable", async () => {
    invoke.mockRejectedValue(new Error("no bridge"));

    render(<FocusProviders />);

    expect(await screen.findByText("web preview")).toBeTruthy();
  });
});
