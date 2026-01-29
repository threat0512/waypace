PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS sessions (
  id TEXT PRIMARY KEY,
  start_ts_ms INTEGER NOT NULL,
  end_ts_ms INTEGER NULL,
  planned_duration_sec INTEGER NOT NULL DEFAULT 2700,
  ended_reason TEXT NULL CHECK (ended_reason IN ('manual','timer')),
  capture_enabled INTEGER NOT NULL DEFAULT 0,
  mode TEXT NOT NULL DEFAULT 'leetcode',
  task_title TEXT NULL,
  created_at_ms INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_sessions_start_ts
ON sessions(start_ts_ms);

CREATE TABLE IF NOT EXISTS events (
  id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL,
  ts_ms INTEGER NOT NULL,
  event_type TEXT NOT NULL,
  app_name TEXT NULL,
  window_title TEXT NULL,
  idle_sec INTEGER NULL,
  FOREIGN KEY(session_id) REFERENCES sessions(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_events_session_ts
ON events(session_id, ts_ms);

CREATE TABLE IF NOT EXISTS nudges (
  id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL,
  ts_ms INTEGER NOT NULL,
  nudge_type TEXT NOT NULL,
  payload_json TEXT NOT NULL,
  user_action TEXT NULL,
  FOREIGN KEY(session_id) REFERENCES sessions(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_nudges_session_ts
ON nudges(session_id, ts_ms);

CREATE TABLE IF NOT EXISTS session_notes (
  session_id TEXT PRIMARY KEY,
  task_link TEXT NULL,
  result TEXT NOT NULL CHECK (result IN ('solved','partial','stuck')),
  tag TEXT NOT NULL,
  lesson TEXT NOT NULL,
  next_step TEXT NOT NULL,
  created_at_ms INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL,
  FOREIGN KEY(session_id) REFERENCES sessions(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS integrations (
  provider TEXT PRIMARY KEY CHECK (provider IN ('notion','sheets')),
  enabled INTEGER NOT NULL DEFAULT 0,
  auth_blob TEXT NOT NULL DEFAULT '',
  config_json TEXT NOT NULL DEFAULT '{}',
  created_at_ms INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL
);
