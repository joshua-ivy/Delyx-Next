export const RATINGS: Array<{ key: "story" | "heroic" | "historical"; label: string; hint: string }> = [
  { key: "story", label: "Story", hint: "adventure-novel tone (default)" },
  { key: "heroic", label: "Heroic", hint: "PG-13 war film intensity" },
  { key: "historical", label: "Historical", hint: "honest to the era" },
];

export function describe(problem: unknown): string {
  return problem instanceof Error ? problem.message : String(problem);
}
