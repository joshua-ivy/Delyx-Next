export interface EraScenarioView {
  id: string;
  title: string;
  startDate: string;
  startLocation: string;
  opening: string;
  squadNames: string[];
}

export interface EraPackView {
  id: string;
  title: string;
  gmStyle: string;
  checks: string[];
  scenarios: EraScenarioView[];
}

export interface CampaignCharacterView {
  id: string;
  kind: "player" | "npc";
  name: string;
  role: string;
  status: "active" | "wounded" | "missing" | "dead" | "departed";
  sheetJson: string;
  inventoryJson: string;
  bondsJson: string;
  notes: string;
}

export interface CampaignEventView {
  id: string;
  turnIndex: number;
  kind: string;
  summary: string;
  createdAt: string;
}

export interface CampaignView {
  id: string;
  projectId: string;
  eraPackId: string;
  scenarioId?: string;
  title: string;
  status: "active" | "completed" | "abandoned";
  contentRating: "story" | "heroic" | "historical";
  worldDate: string;
  location: string;
  memorySummary: string;
  createdAt: string;
  updatedAt: string;
  characters: CampaignCharacterView[];
  turnCount: number;
  events: CampaignEventView[];
}

export interface CampaignSnapshotView {
  campaigns: CampaignView[];
}

export interface TurnResolutionView {
  check: string;
  roll: number[];
  stat: number;
  total: number;
  outcome: "success" | "partial" | "setback";
}

export interface CampaignTurnView {
  turnIndex: number;
  playerText: string;
  resolutionJson: string;
  narration: string;
  stateDeltaJson: string;
  qaqcStatus: "pending" | "clean" | "corrected" | "skipped";
  qaqcNotes?: string;
  createdAt: string;
}

export interface CampaignTurnCommitView {
  turn: CampaignTurnView;
  campaign: CampaignView;
}

export interface CampaignCreateInput {
  projectId: string;
  eraPackId: string;
  scenarioId: string;
  playerName: string;
  playerRole: string;
  contentRating: "story" | "heroic" | "historical";
  title?: string;
  /** A check from the era pack; the character rolls it at +2. */
  playerTrait?: string;
}
