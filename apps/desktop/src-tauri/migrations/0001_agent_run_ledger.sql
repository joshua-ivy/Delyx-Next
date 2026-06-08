CREATE TABLE IF NOT EXISTS agent_runs (
  id TEXT PRIMARY KEY NOT NULL,
  thread_id TEXT NOT NULL,
  status TEXT NOT NULL,
  outcome_summary TEXT,
  outcome_evidence_record_ids TEXT NOT NULL DEFAULT '[]',
  outcome_test_artifact_ids TEXT NOT NULL DEFAULT '[]',
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
  source_id TEXT NOT NULL DEFAULT '',
  title TEXT NOT NULL,
  uri TEXT,
  quote TEXT,
  hash TEXT,
  retrieved_at TEXT NOT NULL DEFAULT '',
  relevance_relationship TEXT,
  relevance_score INTEGER,
  relevance_reason TEXT,
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

CREATE TABLE IF NOT EXISTS memory_candidates (
  id TEXT PRIMARY KEY NOT NULL,
  scope TEXT NOT NULL,
  key TEXT NOT NULL,
  value TEXT NOT NULL,
  source_run_id TEXT NOT NULL,
  source_thread_id TEXT NOT NULL,
  status TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS memory_records (
  id TEXT PRIMARY KEY NOT NULL,
  scope TEXT NOT NULL,
  key TEXT NOT NULL,
  value TEXT NOT NULL,
  source_run_id TEXT NOT NULL,
  source_thread_id TEXT NOT NULL,
  supersedes TEXT,
  suppressed INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS skill_manifests (
  id TEXT PRIMARY KEY NOT NULL,
  name TEXT NOT NULL,
  source TEXT NOT NULL,
  source_hash TEXT NOT NULL,
  trust TEXT NOT NULL,
  status TEXT NOT NULL,
  can_run_scripts INTEGER NOT NULL DEFAULT 0,
  can_edit_files INTEGER NOT NULL DEFAULT 0,
  can_use_network INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS automation_contracts (
  id TEXT PRIMARY KEY NOT NULL,
  title TEXT NOT NULL,
  status TEXT NOT NULL,
  scope TEXT NOT NULL,
  allowed_tools_json TEXT NOT NULL,
  active_start_hour INTEGER NOT NULL,
  active_end_hour INTEGER NOT NULL,
  timezone TEXT NOT NULL,
  delivery_targets_json TEXT NOT NULL,
  stop_condition TEXT NOT NULL,
  workspace_fingerprint TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS scheduled_runs (
  id TEXT PRIMARY KEY NOT NULL,
  contract_id TEXT NOT NULL REFERENCES automation_contracts(id) ON DELETE CASCADE,
  status TEXT NOT NULL,
  reason TEXT NOT NULL,
  approval_id TEXT
);

CREATE TABLE IF NOT EXISTS release_profiles (
  id TEXT PRIMARY KEY NOT NULL,
  product_name TEXT NOT NULL,
  version TEXT NOT NULL,
  target_platform TEXT NOT NULL,
  bundle_target TEXT NOT NULL,
  certificate_thumbprint TEXT,
  digest_algorithm TEXT,
  timestamp_url TEXT,
  sign_command TEXT,
  tsp INTEGER NOT NULL DEFAULT 0,
  update_channel TEXT NOT NULL,
  update_published INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS support_bundles (
  id TEXT PRIMARY KEY NOT NULL,
  app_name TEXT NOT NULL,
  version TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  config_summary_json TEXT NOT NULL,
  logs_json TEXT NOT NULL,
  secret_policy TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS test_artifact_records (
  id TEXT PRIMARY KEY NOT NULL,
  run_id TEXT NOT NULL,
  command TEXT NOT NULL,
  cwd TEXT NOT NULL,
  exit_code INTEGER,
  duration_ms INTEGER NOT NULL,
  stdout TEXT NOT NULL,
  stderr TEXT NOT NULL,
  started_at TEXT NOT NULL,
  completed_at TEXT NOT NULL,
  approval_id TEXT,
  status TEXT NOT NULL,
  failure_summary TEXT,
  output_truncated INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS test_parsed_failures (
  artifact_id TEXT NOT NULL REFERENCES test_artifact_records(id) ON DELETE CASCADE,
  failure_index INTEGER NOT NULL,
  id TEXT NOT NULL,
  message TEXT NOT NULL,
  PRIMARY KEY (artifact_id, failure_index)
);

CREATE TABLE IF NOT EXISTS test_exec_events (
  artifact_id TEXT NOT NULL REFERENCES test_artifact_records(id) ON DELETE CASCADE,
  event_index INTEGER NOT NULL,
  kind TEXT NOT NULL,
  message TEXT NOT NULL,
  timestamp_ms INTEGER NOT NULL,
  PRIMARY KEY (artifact_id, event_index)
);

CREATE TABLE IF NOT EXISTS patch_proposal_records (
  id TEXT PRIMARY KEY NOT NULL,
  run_id TEXT NOT NULL,
  approval_id TEXT NOT NULL,
  status TEXT NOT NULL,
  checkpoint_id TEXT
);

CREATE TABLE IF NOT EXISTS patch_proposal_files (
  proposal_id TEXT NOT NULL REFERENCES patch_proposal_records(id) ON DELETE CASCADE,
  file_index INTEGER NOT NULL,
  path TEXT NOT NULL,
  PRIMARY KEY (proposal_id, file_index)
);

CREATE TABLE IF NOT EXISTS patch_diff_lines (
  proposal_id TEXT NOT NULL,
  file_index INTEGER NOT NULL,
  diff_index INTEGER NOT NULL,
  kind TEXT NOT NULL,
  text TEXT NOT NULL,
  PRIMARY KEY (proposal_id, file_index, diff_index),
  FOREIGN KEY (proposal_id, file_index)
    REFERENCES patch_proposal_files(proposal_id, file_index)
    ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS review_report_records (
  id TEXT PRIMARY KEY NOT NULL,
  run_id TEXT NOT NULL,
  mode TEXT NOT NULL,
  decision TEXT NOT NULL,
  risk_summary TEXT NOT NULL,
  test_summary TEXT NOT NULL,
  evidence_summary TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS review_findings (
  report_id TEXT NOT NULL REFERENCES review_report_records(id) ON DELETE CASCADE,
  finding_index INTEGER NOT NULL,
  id TEXT NOT NULL,
  priority TEXT NOT NULL,
  title TEXT NOT NULL,
  detail TEXT NOT NULL,
  risk_label TEXT NOT NULL,
  suggested_fix TEXT NOT NULL,
  file_path TEXT NOT NULL,
  hunk_label TEXT NOT NULL,
  PRIMARY KEY (report_id, finding_index)
);

CREATE TABLE IF NOT EXISTS external_agent_run_records (
  id TEXT PRIMARY KEY NOT NULL,
  run_id TEXT NOT NULL,
  adapter_id TEXT NOT NULL,
  status TEXT NOT NULL,
  scope TEXT NOT NULL,
  terminal_output TEXT NOT NULL,
  diff_summary TEXT,
  review_required INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS external_agent_run_events (
  artifact_id TEXT NOT NULL REFERENCES external_agent_run_records(id) ON DELETE CASCADE,
  event_index INTEGER NOT NULL,
  kind TEXT NOT NULL,
  message TEXT NOT NULL,
  timestamp TEXT NOT NULL,
  PRIMARY KEY (artifact_id, event_index)
);

CREATE TABLE IF NOT EXISTS external_agent_run_tests (
  artifact_id TEXT NOT NULL REFERENCES external_agent_run_records(id) ON DELETE CASCADE,
  test_index INTEGER NOT NULL,
  test_artifact_id TEXT NOT NULL,
  PRIMARY KEY (artifact_id, test_index)
);

CREATE TABLE IF NOT EXISTS research_evidence_records (
  id TEXT PRIMARY KEY NOT NULL,
  run_id TEXT NOT NULL,
  source_kind TEXT NOT NULL,
  title TEXT NOT NULL,
  locator TEXT NOT NULL,
  excerpt TEXT NOT NULL,
  stance TEXT NOT NULL,
  claim_key TEXT NOT NULL
);
