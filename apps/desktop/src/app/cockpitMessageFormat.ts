import type { TaskThread } from "../features/threads/threadTypes";
import { markdownTextToHtml } from "./markdownHtml";

const assistantPreviewLimit = 560;
const longTextLimit = 760;
const userPreviewLimit = 360;

export function threadGoalBlock(thread: TaskThread | undefined) {
  const goal = thread?.goal ?? "Send an instruction below to start a local thread.";
  if (goal.length <= 420) {
    return markdownTextToHtml(goal);
  }
  return `${markdownTextToHtml(excerpt(goal, 300))}
    <details class="deck-disclosure">
      <summary>Full request</summary>
      <div class="deck-raw-text">${markdownTextToHtml(goal)}</div>
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
    return `<div class="msg-brief">${markdownTextToHtml(excerpt(text, userPreviewLimit))}</div>
      <details class="deck-disclosure">
        <summary>Full request</summary>
        <div class="deck-raw-text">${markdownTextToHtml(text)}</div>
      </details>`;
  }
  if (role === "delyx" && text.length > longTextLimit) {
    return `<div class="msg-brief">${markdownTextToHtml(excerpt(text, assistantPreviewLimit))}</div>
      <details class="deck-disclosure">
        <summary>Full model output</summary>
        <div class="deck-raw-text">${markdownTextToHtml(text)}</div>
      </details>`;
  }
  return markdownTextToHtml(text);
}

function excerpt(text: string, maxLength: number) {
  if (text.length <= maxLength) {
    return text;
  }
  const cut = text.slice(0, maxLength).replace(/\s+\S*$/, "");
  return `${cut}...`;
}
