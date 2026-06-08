import type { ReactNode } from "react";

type Block =
  | { kind: "code"; code: string; language: string }
  | { kind: "heading"; text: string }
  | { kind: "ol"; items: string[] }
  | { kind: "p"; text: string }
  | { kind: "ul"; items: string[] };

export function MarkdownMessage({ text }: { text: string }) {
  const blocks = markdownBlocks(text);
  return <div className="mdmsg">{blocks.map(renderBlock)}</div>;
}

function renderBlock(block: Block, index: number) {
  if (block.kind === "heading") {
    return <h3 key={index}>{inlineNodes(block.text)}</h3>;
  }
  if (block.kind === "ul") {
    return <ul key={index}>{block.items.map((item, itemIndex) => <li key={itemIndex}>{inlineNodes(item)}</li>)}</ul>;
  }
  if (block.kind === "ol") {
    return <ol key={index}>{block.items.map((item, itemIndex) => <li key={itemIndex}>{inlineNodes(item)}</li>)}</ol>;
  }
  if (block.kind === "code") {
    return <pre key={index} data-language={block.language}><code>{block.code}</code></pre>;
  }
  return <p key={index}>{inlineNodes(block.text)}</p>;
}

function markdownBlocks(text: string) {
  const lines = text.replace(/\r\n/g, "\n").split("\n");
  const blocks: Block[] = [];
  let index = 0;
  while (index < lines.length) {
    const line = lines[index];
    const trimmed = line.trim();
    if (!trimmed) {
      index += 1;
    } else if (trimmed.startsWith("```")) {
      const language = trimmed.slice(3).trim();
      const code: string[] = [];
      index += 1;
      while (index < lines.length && !lines[index].trim().startsWith("```")) {
        code.push(lines[index]);
        index += 1;
      }
      blocks.push({ code: code.join("\n"), kind: "code", language });
      index += lines[index]?.trim().startsWith("```") ? 1 : 0;
    } else if (/^#{1,3}\s+/.test(trimmed)) {
      blocks.push({ kind: "heading", text: trimmed.replace(/^#{1,3}\s+/, "") });
      index += 1;
    } else if (/^\s*[-*]\s+/.test(line)) {
      const result = collectList(lines, index, /^\s*[-*]\s+/, "ul");
      blocks.push(result.block);
      index = result.index;
    } else if (/^\s*\d+\.\s+/.test(line)) {
      const result = collectList(lines, index, /^\s*\d+\.\s+/, "ol");
      blocks.push(result.block);
      index = result.index;
    } else {
      const paragraph: string[] = [];
      while (index < lines.length && isParagraphLine(lines[index])) {
        paragraph.push(lines[index].trim());
        index += 1;
      }
      blocks.push({ kind: "p", text: paragraph.join(" ") });
    }
  }
  return blocks;
}

function collectList(lines: string[], start: number, marker: RegExp, kind: "ol" | "ul") {
  const items: string[] = [];
  let index = start;
  while (index < lines.length && marker.test(lines[index])) {
    items.push(lines[index].replace(marker, "").trim());
    index += 1;
  }
  return { block: { items, kind } as Block, index };
}

function isParagraphLine(line: string) {
  const trimmed = line.trim();
  return Boolean(trimmed)
    && !trimmed.startsWith("```")
    && !/^#{1,3}\s+/.test(trimmed)
    && !/^\s*[-*]\s+/.test(line)
    && !/^\s*\d+\.\s+/.test(line);
}

function inlineNodes(text: string) {
  const nodes: ReactNode[] = [];
  let index = 0;
  while (index < text.length) {
    const codeStart = text.indexOf("`", index);
    const boldStart = text.indexOf("**", index);
    const next = nextMarker(codeStart, boldStart, text.length);
    if (next > index) {
      nodes.push(text.slice(index, next));
      index = next;
    } else if (codeStart === index) {
      index = pushDelimited(nodes, text, index, "`", "code");
    } else if (boldStart === index) {
      index = pushDelimited(nodes, text, index, "**", "strong");
    } else {
      nodes.push(text[index]);
      index += 1;
    }
  }
  return nodes;
}

function nextMarker(codeStart: number, boldStart: number, fallback: number) {
  return Math.min(codeStart === -1 ? fallback : codeStart, boldStart === -1 ? fallback : boldStart);
}

function pushDelimited(nodes: ReactNode[], text: string, index: number, token: "`" | "**", tag: "code" | "strong") {
  const end = text.indexOf(token, index + token.length);
  if (end === -1) {
    nodes.push(token);
    return index + token.length;
  }
  const body = text.slice(index + token.length, end);
  nodes.push(tag === "code" ? <code key={index}>{body}</code> : <strong key={index}>{body}</strong>);
  return end + token.length;
}
