import { readFileSync } from "node:fs";
import { join } from "node:path";
import vm from "node:vm";
import { fileURLToPath } from "node:url";
import ts from "typescript";

const root = fileURLToPath(new URL("..", import.meta.url));
const markdown = loadMarkdownModule();
const failures = [];

const sample = [
  "**Web Technologies (Most Universal)**",
  "- **TypeScript + React/Vue/Svelte**: Best for cross-platform web apps",
  "- `HTML/CSS/JavaScript`: Direct browser control",
  "",
  "1. **Rust + egui/Tauri**: Excellent desktop performance",
  "",
  "```rust",
  "fn main() {}",
  "```",
].join("\n");
const html = markdown.markdownTextToHtml(sample);

check(html.includes("<strong>Web Technologies (Most Universal)</strong>"), "bold paragraph text must render");
check(html.includes("<li><strong>TypeScript + React/Vue/Svelte</strong>: Best"), "bold unordered list text must render");
check(html.includes("<code>HTML/CSS/JavaScript</code>"), "inline code must render");
check(html.includes("<ol><li><strong>Rust + egui/Tauri</strong>: Excellent"), "ordered list text must render");
check(html.includes('<pre class="msg-code" data-language="rust"><code>fn main() {}</code></pre>'), "fenced code must render");
check(!html.includes("**TypeScript"), "rendered HTML must not leak raw bold tokens");
check(markdown.markdownTextToHtml("<script>x</script>").includes("&lt;script&gt;x&lt;/script&gt;"), "raw HTML must stay escaped");
check(!markdown.markdownTextToHtml("[bad](javascript:alert(1))").includes("href=\"javascript:"), "unsafe links must not render as anchors");

if (failures.length > 0) {
  console.error(failures.join("\n"));
  process.exit(1);
}

console.log("Markdown render verifier passed: assistant messages format safe Markdown.");

function loadMarkdownModule() {
  const source = readFileSync(join(root, "src/app/markdownHtml.ts"), "utf8");
  const compiled = ts.transpileModule(source, {
    compilerOptions: { esModuleInterop: true, module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 },
  }).outputText;
  const module = { exports: {} };
  const sandbox = {
    exports: module.exports,
    module,
    require(id) {
      if (id === "./html") {
        return { escapeHtml };
      }
      throw new Error(`Unexpected test import: ${id}`);
    },
  };
  vm.runInNewContext(compiled, sandbox, { filename: "markdownHtml.ts" });
  return module.exports;
}

function escapeHtml(value) {
  return value.replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;").replaceAll('"', "&quot;");
}

function check(condition, message) {
  if (!condition) {
    failures.push(message);
  }
}
