export interface RunnableTestCommand {
  args: string[];
  label: string;
  program: string;
}

export function firstRunnableTestCommand(commands: string[] | undefined) {
  return (commands ?? []).map(parseRunnableTestCommand).find((command) => !!command);
}

export function parseRunnableTestCommand(command: string): RunnableTestCommand | undefined {
  const label = command.trim();
  if (!label || hasShellControl(label)) {
    return undefined;
  }
  const parts = splitCommand(label);
  const [program, ...args] = parts;
  const name = normalizedProgram(program);
  if (!program || !isSupportedTestCommand(name, args)) {
    return undefined;
  }
  return { args, label, program };
}

function hasShellControl(command: string) {
  return /(?:&&|\|\||[;|<>`])/.test(command);
}

function splitCommand(command: string) {
  const parts: string[] = [];
  let current = "";
  let quote: string | undefined;
  for (const char of command) {
    if ((char === "\"" || char === "'") && !quote) {
      quote = char;
    } else if (char === quote) {
      quote = undefined;
    } else if (/\s/.test(char) && !quote) {
      pushPart(parts, current);
      current = "";
    } else {
      current += char;
    }
  }
  if (quote) {
    return [];
  }
  pushPart(parts, current);
  return parts;
}

function pushPart(parts: string[], value: string) {
  const trimmed = value.trim();
  if (trimmed) {
    parts.push(trimmed);
  }
}

function normalizedProgram(program: string | undefined) {
  const base = (program ?? "").replace(/\\/g, "/").split("/").pop() ?? "";
  return base.toLowerCase().replace(/\.(cmd|bat|exe)$/, "");
}

function isSupportedTestCommand(program: string, args: string[]) {
  if (program === "cargo") {
    return args[0] === "test" || args[0] === "nextest";
  }
  if (program === "npm" || program === "pnpm" || program === "yarn") {
    return args[0] === "test" || (args[0] === "run" && args[1] === "test");
  }
  return program === "pytest" || program === "vitest" || program === "cargo-nextest";
}
