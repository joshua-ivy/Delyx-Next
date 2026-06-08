import type { TaskThread } from "../features/threads/threadTypes";
import { escapeHtml } from "./html";

const assistantPreviewLimit = 560;
const longTextLimit = 760;
const userPreviewLimit = 360;

export function threadGoalBlock(thread: TaskThread | undefined) {
  const goal = thread?.goal ?? "Send an instruction below to start a local thread.";
  if (goal.length <= 420) {
    return formatPlainText(goal);
  }
  return `${formatPlainText(excerpt(goal, 300))}
    <details class="deck-disclosure">
      <summary>Full request</summary>
      <div class="deck-raw-text">${formatPlainText(goal)}</div>
    </details>`;
}

export function conversationBlock(thread: TaskThread | undefined) {
  if (!thread) {
    return "";
  }
  return thread.messages.map(messageBlock).join("");
}

function messageBlock(message: TaskThread["messages"][number]) {
  const role = message.role === "user" ? "you" : message.role === "assistant" ? "delyx" : "system";
  const avatar = role === "delyx" ? '<span class="deck-msg-av">D</span>' : "";
  return `<div class="deck-msg ${role}">${avatar}<div class="deck-msg-bub">${messageBody(message.body, role)}</div></div>`;
}

function messageBody(body: string, role: "you" | "delyx" | "system") {
  const text = body.trim();
  if (role === "you" && text.length > longTextLimit) {
    return `<div class="msg-brief">${formatPlainText(excerpt(text, userPreviewLimit))}</div>
      <details class="deck-disclosure">
        <summary>Full request</summary>
        <div class="deck-raw-text">${formatPlainText(text)}</div>
      </details>`;
  }
  if (role === "delyx" && text.length > longTextLimit) {
    return `<div class="msg-brief">${formatRichText(excerpt(text, assistantPreviewLimit))}</div>
      <details class="deck-disclosure">
        <summary>Full model output</summary>
        <div class="deck-raw-text">${formatRichText(text)}</div>
      </details>`;
  }
  return formatRichText(text);
}

function formatRichText(text: string) {
  return splitCodeFences(text).map((block) => {
    if (block.kind === "code") {
      return `<pre class="msg-code"><code>${escapeHtml(block.text.trim())}</code></pre>`;
    }
    return formatPlainText(block.text);
  }).join("");
}

function formatPlainText(text: string) {
  const lines = text.replace(/\r\n/g, "\n").split("\n");
  const blocks: string[] = [];
  let list: string[] = [];
  let paragraph: string[] = [];

  const flushParagraph = () => {
    if (paragraph.length > 0) {
      blocks.push(`<p>${escapeHtml(paragraph.join(" ").trim())}</p>`);
      paragraph = [];
    }
  };
  const flushList = () => {
    if (list.length > 0) {
      blocks.push(`<ul>${list.map((item) => `<li>${escapeHtml(item)}</li>`).join("")}</ul>`);
      list = [];
    }
  };

  for (const line of lines) {
    const trimmed = line.trim();
    if (!trimmed) {
      flushParagraph();
      flushList();
      continue;
    }
    const heading = trimmed.match(/^#{1,3}\s+(.+)$/);
    const item = trimmed.match(/^[-*]\s+(.+)$/) ?? trimmed.match(/^\d+[.)]\s+(.+)$/);
    if (heading) {
      flushParagraph();
      flushList();
      blocks.push(`<h3>${escapeHtml(heading[1])}</h3>`);
    } else if (item) {
      flushParagraph();
      list.push(item[1]);
    } else {
      flushList();
      paragraph.push(trimmed);
    }
  }
  flushParagraph();
  flushList();
  return blocks.join("");
}

function splitCodeFences(text: string) {
  const blocks: { kind: "text" | "code"; text: string }[] = [];
  const pattern = /```[\w-]*\n([\s\S]*?)```/g;
  let last = 0;
  let match: RegExpExecArray | null;
  while ((match = pattern.exec(text)) !== null) {
    blocks.push({ kind: "text", text: text.slice(last, match.index) });
    blocks.push({ kind: "code", text: match[1] });
    last = pattern.lastIndex;
  }
  blocks.push({ kind: "text", text: text.slice(last) });
  return blocks.filter((block) => block.text.trim());
}

function excerpt(text: string, maxLength: number) {
  if (text.length <= maxLength) {
    return text;
  }
  const cut = text.slice(0, maxLength).replace(/\s+\S*$/, "");
  return `${cut}...`;
}
