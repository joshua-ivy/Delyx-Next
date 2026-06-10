CREATE TABLE IF NOT EXISTS campaigns (
  id TEXT PRIMARY KEY NOT NULL,
  project_id TEXT NOT NULL,
  era_pack_id TEXT NOT NULL,
  scenario_id TEXT,
  title TEXT NOT NULL,
  status TEXT NOT NULL,
  content_rating TEXT NOT NULL,
  world_date TEXT NOT NULL,
  location TEXT NOT NULL,
  memory_summary TEXT NOT NULL DEFAULT '',
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_campaigns_project
  ON campaigns(project_id);

CREATE TABLE IF NOT EXISTS campaign_characters (
  id TEXT PRIMARY KEY NOT NULL,
  campaign_id TEXT NOT NULL REFERENCES campaigns(id) ON DELETE CASCADE,
  kind TEXT NOT NULL,
  name TEXT NOT NULL,
  role TEXT NOT NULL,
  status TEXT NOT NULL,
  sheet_json TEXT NOT NULL DEFAULT '{}',
  inventory_json TEXT NOT NULL DEFAULT '[]',
  bonds_json TEXT NOT NULL DEFAULT '[]',
  notes TEXT NOT NULL DEFAULT '',
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_campaign_characters_campaign
  ON campaign_characters(campaign_id);

CREATE TABLE IF NOT EXISTS campaign_turns (
  campaign_id TEXT NOT NULL REFERENCES campaigns(id) ON DELETE CASCADE,
  turn_index INTEGER NOT NULL,
  player_text TEXT NOT NULL,
  resolution_json TEXT NOT NULL DEFAULT '{}',
  narration TEXT NOT NULL,
  state_delta_json TEXT NOT NULL DEFAULT '{}',
  qaqc_status TEXT NOT NULL DEFAULT 'pending',
  qaqc_notes TEXT,
  created_at TEXT NOT NULL,
  PRIMARY KEY (campaign_id, turn_index)
);

CREATE TABLE IF NOT EXISTS campaign_events (
  id TEXT PRIMARY KEY NOT NULL,
  campaign_id TEXT NOT NULL REFERENCES campaigns(id) ON DELETE CASCADE,
  turn_index INTEGER NOT NULL,
  kind TEXT NOT NULL,
  summary TEXT NOT NULL,
  created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_campaign_events_campaign
  ON campaign_events(campaign_id);
