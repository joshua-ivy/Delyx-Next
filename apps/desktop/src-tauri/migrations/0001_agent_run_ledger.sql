CREATE TABLE IF NOT EXISTS agent_runs (
  id TEXT PRIMARY KEY NOT NULL,
  thread_id TEXT NOT NULL,
  status TEXT NOT NULL,
  outcome_summary TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS agent_nodes (
  id TEXT PRIMARY KEY NOT NULL,
  run_id TEXT NOT NULL REFERENCES agent_runs(id) ON DELETE CASCADE,
  kind TEXT NOT NULL,
  label TEXT NOT NULL,
  status TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS agent_events (
  id TEXT PRIMARY KEY NOT NULL,
  run_id TEXT NOT NULL REFERENCES agent_runs(id) ON DELETE CASCADE,
  kind TEXT NOT NULL,
  message TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS artifacts (
  id TEXT NOT NULL,
  run_id TEXT NOT NULL REFERENCES agent_runs(id) ON DELETE CASCADE,
  kind TEXT NOT NULL,
  label TEXT NOT NULL,
  PRIMARY KEY (run_id, id)
);

CREATE TABLE IF NOT EXISTS evidence_records (
  id TEXT NOT NULL,
  run_id TEXT NOT NULL REFERENCES agent_runs(id) ON DELETE CASCADE,
  source_kind TEXT NOT NULL,
  title TEXT NOT NULL,
  PRIMARY KEY (run_id, id)
);

CREATE TABLE IF NOT EXISTS task_threads (
  id TEXT PRIMARY KEY NOT NULL,
  project_id TEXT NOT NULL,
  title TEXT NOT NULL,
  goal TEXT NOT NULL,
  status TEXT NOT NULL,
  archived INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS thread_messages (
  thread_id TEXT NOT NULL REFERENCES task_threads(id) ON DELETE CASCADE,
  message_index INTEGER NOT NULL,
  role TEXT NOT NULL,
  body TEXT NOT NULL,
  PRIMARY KEY (thread_id, message_index)
);

CREATE TABLE IF NOT EXISTS thread_run_records (
  thread_id TEXT PRIMARY KEY NOT NULL REFERENCES task_threads(id) ON DELETE CASCADE,
  run_id TEXT NOT NULL REFERENCES agent_runs(id) ON DELETE CASCADE,
  project_id TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS action_proposals (
  id TEXT PRIMARY KEY NOT NULL,
  run_id TEXT NOT NULL,
  node_id TEXT NOT NULL,
  action TEXT NOT NULL,
  risk TEXT NOT NULL,
  scope TEXT NOT NULL,
  reason TEXT NOT NULL,
  expected_result TEXT NOT NULL,
  rollback_plan TEXT NOT NULL,
  expires_at INTEGER NOT NULL,
  status TEXT NOT NULL,
  decision_kind TEXT,
  decision_at INTEGER,
  decision_note TEXT
);

CREATE TABLE IF NOT EXISTS approval_bridge_records (
  client_id TEXT PRIMARY KEY NOT NULL,
  proposal_id TEXT NOT NULL REFERENCES action_proposals(id) ON DELETE CASCADE,
  run_id TEXT NOT NULL,
  action_type TEXT NOT NULL,
  required_permission TEXT NOT NULL,
  expires_at TEXT NOT NULL,
  scope_json TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS workspace_project_snapshots (
  id TEXT PRIMARY KEY NOT NULL,
  path TEXT NOT NULL,
  project_json TEXT NOT NULL,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS model_role_routes (
  role TEXT PRIMARY KEY NOT NULL,
  provider_id TEXT NOT NULL,
  model_id TEXT NOT NULL,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
