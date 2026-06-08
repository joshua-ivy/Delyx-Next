import { escapeHtml } from "./html";

type CodeFenceBlock = { kind: "code"; language: string; text: string } | { kind: "text"; text: string };
type ListKind = "ol" | "ul";

export function markdownTextToHtml(text: string) {
  return splitCodeFences(text).map((block) => {
    if (block.kind === "code") {
      const language = block.language ? ` data-language="${escapeHtml(block.language)}"` : "";
      return `<pre class="msg-code"${language}><code>${escapeHtml(block.text.trim())}</code></pre>`;
    }
    return markdownBlocksToHtml(block.text);
  }).join("");
}

export function inlineMarkdownToHtml(text: string): string {
  const chunks: string[] = [];
  let index = 0;
  while (index < text.length) {
    const marker = nextInlineMarker(text, index);
    if (marker.index > index) {
      chunks.push(escapeHtml(text.slice(index, marker.index)));
      index = marker.index;
    } else if (marker.kind === "code") {
      index = pushInlineCode(chunks, text, index);
    } else if (marker.kind === "strong") {
      index = pushStrong(chunks, text, index);
    } else if (marker.kind === "link") {
      index = pushLink(chunks, text, index);
    } else {
      chunks.push(escapeHtml(text[index]));
      index += 1;
    }
  }
  return chunks.join("");
}

function markdownBlocksToHtml(text: string) {
  const lines = text.replace(/\r\n/g, "\n").split("\n");
  const blocks: string[] = [];
  let list: { items: string[]; kind: ListKind } | undefined;
  let paragraph: string[] = [];

  const flushParagraph = () => {
    if (paragraph.length > 0) {
      blocks.push(`<p>${inlineMarkdownToHtml(paragraph.join(" ").trim())}</p>`);
      paragraph = [];
    }
  };
  const flushList = () => {
    if (list) {
      blocks.push(`<${list.kind}>${list.items.map((item) => `<li>${inlineMarkdownToHtml(item)}</li>`).join("")}</${list.kind}>`);
      list = undefined;
    }
  };

  for (const line of lines) {
    const trimmed = line.trim();
    if (!trimmed) {
      flushParagraph();
      flushList();
      continue;
    }
    const heading = trimmed.match(/^#{1,6}\s+(.+)$/);
    const unordered = trimmed.match(/^[-*+]\s+(.+)$/);
    const ordered = trimmed.match(/^\d+[.)]\s+(.+)$/);
    const quote = trimmed.match(/^>\s?(.+)$/);
    if (heading) {
      flushParagraph();
      flushList();
      blocks.push(`<h3>${inlineMarkdownToHtml(heading[1])}</h3>`);
    } else if (unordered || ordered) {
      flushParagraph();
      const kind: ListKind = ordered ? "ol" : "ul";
      if (list?.kind !== kind) {
        flushList();
        list = { items: [], kind };
      }
      list.items.push((unordered ?? ordered)?.[1] ?? "");
    } else if (quote) {
      flushParagraph();
      flushList();
      blocks.push(`<blockquote>${inlineMarkdownToHtml(quote[1])}</blockquote>`);
    } else {
      flushList();
      paragraph.push(trimmed);
    }
  }
  flushParagraph();
  flushList();
  return blocks.join("");
}

function nextInlineMarker(text: string, start: number) {
  const markers = [
    { index: text.indexOf("`", start), kind: "code" as const },
    { index: text.indexOf("**", start), kind: "strong" as const },
    { index: text.indexOf("[", start), kind: "link" as const },
  ].filter((marker) => marker.index !== -1);
  return markers.sort((a, b) => a.index - b.index)[0] ?? { index: text.length, kind: "text" as const };
}

function pushInlineCode(chunks: string[], text: string, index: number) {
  const end = text.indexOf("`", index + 1);
  if (end === -1) {
    chunks.push("`");
    return index + 1;
  }
  chunks.push(`<code>${escapeHtml(text.slice(index + 1, end))}</code>`);
  return end + 1;
}

function pushStrong(chunks: string[], text: string, index: number) {
  const end = text.indexOf("**", index + 2);
  if (end === -1) {
    chunks.push("**");
    return index + 2;
  }
  chunks.push(`<strong>${inlineMarkdownToHtml(text.slice(index + 2, end))}</strong>`);
  return end + 2;
}

function pushLink(chunks: string[], text: string, index: number) {
  const labelEnd = text.indexOf("]", index + 1);
  const urlStart = labelEnd === -1 ? -1 : labelEnd + 1;
  const urlEnd = urlStart === -1 || text[urlStart] !== "(" ? -1 : text.indexOf(")", urlStart + 1);
  if (labelEnd === -1 || urlEnd === -1) {
    chunks.push("[");
    return index + 1;
  }
  const href = text.slice(urlStart + 1, urlEnd).trim();
  if (!safeHref(href)) {
    chunks.push(escapeHtml(text.slice(index, urlEnd + 1)));
    return urlEnd + 1;
  }
  chunks.push(`<a href="${escapeHtml(href)}" rel="noreferrer" target="_blank">${inlineMarkdownToHtml(text.slice(index + 1, labelEnd))}</a>`);
  return urlEnd + 1;
}

function safeHref(href: string) {
  return /^(https?:|mailto:)/i.test(href);
}

function splitCodeFences(text: string): CodeFenceBlock[] {
  const blocks: CodeFenceBlock[] = [];
  const pattern = /```([\w-]*)\n([\s\S]*?)```/g;
  let last = 0;
  let match: RegExpExecArray | null;
  while ((match = pattern.exec(text)) !== null) {
    blocks.push({ kind: "text", text: text.slice(last, match.index) });
    blocks.push({ kind: "code", language: match[1].trim(), text: match[2] });
    last = pattern.lastIndex;
  }
  blocks.push({ kind: "text", text: text.slice(last) });
  return blocks.filter((block) => block.text.trim());
}
