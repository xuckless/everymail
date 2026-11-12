-- The local replica. Instants are Unix seconds. A column of codes names the enum whose codec in
-- engine/src/store/codes.rs reads and writes it; ui.v1 enums are stored as their proto numbers.
-- A nullable column is NULL where its comment says the value does not apply. No secret is stored.

-- One row once signed in.
CREATE TABLE session (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  user_name TEXT NOT NULL,
  -- world::Identity
  provider INTEGER NOT NULL
);

-- ui.v1 Theme, Density and Sidebar.
CREATE TABLE preferences (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  theme INTEGER NOT NULL,
  density INTEGER NOT NULL,
  sidebar INTEGER NOT NULL
);

-- The navigation's web path, reopened on launch.
CREATE TABLE navigation (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  path TEXT NOT NULL
);

CREATE TABLE accounts (
  id TEXT PRIMARY KEY,
  position INTEGER NOT NULL,
  name TEXT NOT NULL,
  address TEXT NOT NULL,
  -- ui.v1 MailProvider
  provider INTEGER NOT NULL,
  provider_name TEXT NOT NULL,
  -- AccountKind
  state INTEGER NOT NULL,
  -- Syncing only: the first sync's progress
  sync_percent INTEGER,
  -- Sign-in expired only: "App password expired"
  expired_reason TEXT,
  synced_at INTEGER NOT NULL,
  -- NULL: no signature
  signature TEXT
);

CREATE TABLE folders (
  id TEXT PRIMARY KEY,
  position INTEGER NOT NULL,
  name TEXT NOT NULL,
  -- world::FolderKind
  kind INTEGER NOT NULL
);

CREATE TABLE threads (
  id TEXT PRIMARY KEY,
  account_id TEXT NOT NULL REFERENCES accounts (id) ON DELETE CASCADE,
  -- world::Place
  place INTEGER NOT NULL,
  -- ui.v1 InboxCategory
  category INTEGER NOT NULL,
  sender TEXT NOT NULL,
  subject TEXT NOT NULL,
  latest_at INTEGER NOT NULL,
  -- What an agent did with it, and ui.v1 NoteKind; both NULL without a note
  note_text TEXT,
  note_kind INTEGER,
  -- Scribe's summary; NULL until written
  summary TEXT,
  unread INTEGER NOT NULL,
  -- Out of the inbox until then
  snoozed_until INTEGER,
  CHECK ((note_text IS NULL) = (note_kind IS NULL))
);

CREATE TABLE thread_folders (
  thread_id TEXT NOT NULL REFERENCES threads (id) ON DELETE CASCADE,
  folder_id TEXT NOT NULL REFERENCES folders (id) ON DELETE CASCADE,
  PRIMARY KEY (thread_id, folder_id)
);

-- The people besides the user and the sender on every message of a thread.
CREATE TABLE thread_people (
  thread_id TEXT NOT NULL REFERENCES threads (id) ON DELETE CASCADE,
  position INTEGER NOT NULL,
  name TEXT NOT NULL,
  PRIMARY KEY (thread_id, position)
);

-- Scribe's suggested replies.
CREATE TABLE quick_replies (
  thread_id TEXT NOT NULL REFERENCES threads (id) ON DELETE CASCADE,
  position INTEGER NOT NULL,
  text TEXT NOT NULL,
  PRIMARY KEY (thread_id, position)
);

CREATE TABLE messages (
  id TEXT PRIMARY KEY,
  thread_id TEXT NOT NULL REFERENCES threads (id) ON DELETE CASCADE,
  position INTEGER NOT NULL,
  from_you INTEGER NOT NULL,
  sender TEXT NOT NULL,
  preview TEXT NOT NULL,
  sent_at INTEGER NOT NULL,
  -- Paragraphs separated by a blank line
  body TEXT NOT NULL,
  -- The event this message invites to
  invitation_event TEXT,
  starred INTEGER NOT NULL
);

CREATE TABLE attachments (
  id TEXT PRIMARY KEY,
  message_id TEXT NOT NULL REFERENCES messages (id) ON DELETE CASCADE,
  position INTEGER NOT NULL,
  name TEXT NOT NULL,
  size_bytes INTEGER NOT NULL,
  -- ui.v1 AttachmentKind
  kind INTEGER NOT NULL
);

-- Local search, written in the same transaction as the message row.
CREATE VIRTUAL TABLE mail_search USING fts5 (
  message_id UNINDEXED,
  thread_id UNINDEXED,
  subject,
  sender,
  recipients,
  body,
  tokenize = 'unicode61 remove_diacritics 2'
);

CREATE TABLE agents (
  id TEXT PRIMARY KEY,
  position INTEGER NOT NULL,
  name TEXT NOT NULL,
  -- A system agent's fixed instructions; NULL for an agent the user made
  built_in TEXT,
  active INTEGER NOT NULL,
  -- The user's instructions: appended to a system agent's, the whole of a user agent's
  instructions TEXT NOT NULL,
  model_name TEXT NOT NULL,
  -- ui.v1 ModelProvider
  model_provider INTEGER NOT NULL,
  -- ScheduleKind
  schedule INTEGER NOT NULL,
  -- Built-in schedules only: how the Backend's order on new mail reaches it
  schedule_note TEXT,
  -- Set times only: weekdays or every day; the times are agent_times
  weekdays_only INTEGER,
  -- Interval schedules only
  interval_seconds INTEGER,
  -- When its schedule last woke it, or when it was made: the next wake is the first slot after
  woken_at INTEGER NOT NULL,
  -- What it does without a model key, as the Backend says it
  key_note TEXT NOT NULL,
  -- world::ResultsKind
  results_kind INTEGER NOT NULL,
  has_chat INTEGER NOT NULL
);

CREATE TABLE agent_times (
  agent_id TEXT NOT NULL REFERENCES agents (id) ON DELETE CASCADE,
  -- Minutes after midnight
  minute INTEGER NOT NULL,
  PRIMARY KEY (agent_id, minute)
);

-- What wakes an agent besides its schedule: builder::TRIGGERS ids.
CREATE TABLE agent_triggers (
  agent_id TEXT NOT NULL REFERENCES agents (id) ON DELETE CASCADE,
  trigger_id TEXT NOT NULL,
  PRIMARY KEY (agent_id, trigger_id)
);

-- The filter choices picked, by id; an optional filter without a row takes its default.
CREATE TABLE agent_filters (
  agent_id TEXT NOT NULL REFERENCES agents (id) ON DELETE CASCADE,
  trigger_id TEXT NOT NULL,
  filter_id TEXT NOT NULL,
  choice_id TEXT NOT NULL,
  PRIMARY KEY (agent_id, trigger_id, filter_id),
  FOREIGN KEY (agent_id, trigger_id) REFERENCES agent_triggers (agent_id, trigger_id) ON DELETE CASCADE
);

-- The actions agents may take, in the order controls list them.
CREATE TABLE actions (
  id TEXT PRIMARY KEY,
  position INTEGER NOT NULL,
  label TEXT NOT NULL,
  hint TEXT,
  -- Offered to a new agent in the builder
  offered INTEGER NOT NULL
);

-- ui.v1 PermissionLevel per action: an agent's, the defaults for new agents, and the builder's,
-- with the levels the Backend offers: ask_at_most is why Auto is not offered, NULL when it is.
CREATE TABLE agent_permissions (
  agent_id TEXT NOT NULL REFERENCES agents (id) ON DELETE CASCADE,
  action_id TEXT NOT NULL REFERENCES actions (id),
  level INTEGER NOT NULL,
  ask_at_most TEXT,
  PRIMARY KEY (agent_id, action_id)
);

CREATE TABLE default_permissions (
  action_id TEXT PRIMARY KEY REFERENCES actions (id),
  level INTEGER NOT NULL,
  ask_at_most TEXT
);

CREATE TABLE builder_permissions (
  action_id TEXT PRIMARY KEY REFERENCES actions (id),
  level INTEGER NOT NULL,
  ask_at_most TEXT
);

CREATE TABLE agent_accounts (
  agent_id TEXT NOT NULL REFERENCES agents (id) ON DELETE CASCADE,
  account_id TEXT NOT NULL REFERENCES accounts (id) ON DELETE CASCADE,
  PRIMARY KEY (agent_id, account_id)
);

CREATE TABLE agent_stats (
  agent_id TEXT NOT NULL REFERENCES agents (id) ON DELETE CASCADE,
  position INTEGER NOT NULL,
  value TEXT NOT NULL,
  label TEXT NOT NULL,
  PRIMARY KEY (agent_id, position)
);

-- Counts shown over an agent's results.
CREATE TABLE result_stats (
  agent_id TEXT NOT NULL REFERENCES agents (id) ON DELETE CASCADE,
  position INTEGER NOT NULL,
  value TEXT NOT NULL,
  label TEXT NOT NULL,
  PRIMARY KEY (agent_id, position)
);

CREATE TABLE result_items (
  id TEXT PRIMARY KEY,
  agent_id TEXT NOT NULL REFERENCES agents (id) ON DELETE CASCADE,
  group_title TEXT NOT NULL,
  group_position INTEGER NOT NULL,
  position INTEGER NOT NULL,
  at INTEGER NOT NULL,
  title TEXT NOT NULL,
  detail TEXT NOT NULL
);

CREATE TABLE runs (
  id TEXT PRIMARY KEY,
  agent_id TEXT NOT NULL REFERENCES agents (id) ON DELETE CASCADE,
  started_at INTEGER NOT NULL,
  -- CauseKind
  cause INTEGER NOT NULL,
  -- By cause: the schedule's label, the turn id, the handing agent, the account's name, the
  -- category or folder id, the finished run's id or the lead time id; NULL for the others. The
  -- cause's references are cause_refs.
  cause_text TEXT,
  -- How long it ran; NULL while it runs
  seconds INTEGER,
  tokens INTEGER NOT NULL,
  -- OutcomeKind
  outcome INTEGER NOT NULL,
  -- Failed or stopped only: why
  outcome_text TEXT,
  summary TEXT NOT NULL
);

CREATE TABLE run_steps (
  run_id TEXT NOT NULL REFERENCES runs (id) ON DELETE CASCADE,
  position INTEGER NOT NULL,
  verb TEXT NOT NULL,
  what TEXT NOT NULL,
  -- ui.v1 StepPermission
  permission INTEGER NOT NULL,
  PRIMARY KEY (run_id, position)
);

CREATE TABLE turns (
  id TEXT PRIMARY KEY,
  agent_id TEXT NOT NULL REFERENCES agents (id) ON DELETE CASCADE,
  sent_at INTEGER NOT NULL,
  message TEXT NOT NULL,
  -- TurnKind
  state INTEGER NOT NULL,
  -- The running label, the answer's paragraphs, the failure or the stop; NULL while waiting
  text TEXT,
  -- Waiting only: the approval it is parked on
  approval_id TEXT
);

-- What a result item, a run's cause, a step or a chat answer is about, deleted with it. kind is
-- RefKind.
CREATE TABLE item_refs (
  item_id TEXT NOT NULL REFERENCES result_items (id) ON DELETE CASCADE,
  position INTEGER NOT NULL,
  kind INTEGER NOT NULL,
  target TEXT NOT NULL,
  PRIMARY KEY (item_id, position)
);

CREATE TABLE cause_refs (
  run_id TEXT NOT NULL REFERENCES runs (id) ON DELETE CASCADE,
  position INTEGER NOT NULL,
  kind INTEGER NOT NULL,
  target TEXT NOT NULL,
  PRIMARY KEY (run_id, position)
);

CREATE TABLE step_refs (
  run_id TEXT NOT NULL,
  step INTEGER NOT NULL,
  position INTEGER NOT NULL,
  kind INTEGER NOT NULL,
  target TEXT NOT NULL,
  PRIMARY KEY (run_id, step, position),
  FOREIGN KEY (run_id, step) REFERENCES run_steps (run_id, position) ON DELETE CASCADE
);

CREATE TABLE turn_refs (
  turn_id TEXT NOT NULL REFERENCES turns (id) ON DELETE CASCADE,
  position INTEGER NOT NULL,
  kind INTEGER NOT NULL,
  target TEXT NOT NULL,
  PRIMARY KEY (turn_id, position)
);

CREATE TABLE approvals (
  id TEXT PRIMARY KEY,
  agent_id TEXT NOT NULL REFERENCES agents (id) ON DELETE CASCADE,
  created_at INTEGER NOT NULL,
  title TEXT NOT NULL,
  detail TEXT NOT NULL,
  reason TEXT NOT NULL,
  -- ApprovalKind
  action INTEGER NOT NULL,
  -- A send only: the thread it answers
  thread_id TEXT,
  -- StatusKind
  status INTEGER NOT NULL,
  -- The drafted mail of a send, paragraphs separated by a blank line
  draft TEXT NOT NULL,
  -- Declined only: why
  declined_reason TEXT
);

-- Mail the user is writing, saved on a debounce.
CREATE TABLE drafts (
  id TEXT PRIMARY KEY,
  -- world::DraftKind
  kind INTEGER NOT NULL,
  thread_id TEXT REFERENCES threads (id) ON DELETE CASCADE,
  -- The message replied to or forwarded
  message_id TEXT REFERENCES messages (id) ON DELETE CASCADE,
  -- The approval whose drafted reply this is; sending it approves the item
  approval_id TEXT REFERENCES approvals (id) ON DELETE SET NULL,
  account_id TEXT NOT NULL REFERENCES accounts (id) ON DELETE CASCADE,
  recipients TEXT NOT NULL,
  subject TEXT NOT NULL,
  body TEXT NOT NULL,
  updated_at INTEGER NOT NULL
);

CREATE TABLE draft_attachments (
  id TEXT PRIMARY KEY,
  draft_id TEXT NOT NULL REFERENCES drafts (id) ON DELETE CASCADE,
  position INTEGER NOT NULL,
  name TEXT NOT NULL,
  size_bytes INTEGER NOT NULL
);

-- A send handed to the Backend, in the state the Backend reports.
CREATE TABLE outbox (
  draft_id TEXT PRIMARY KEY REFERENCES drafts (id) ON DELETE CASCADE,
  -- SendKind
  state INTEGER NOT NULL,
  -- Undoable and scheduled only: when the undo window closes, or when it sends
  due_at INTEGER,
  -- Failed only: why
  failure TEXT
);

CREATE TABLE calendars (
  id TEXT PRIMARY KEY,
  position INTEGER NOT NULL,
  name TEXT NOT NULL,
  -- ui.v1 CalendarProvider
  provider INTEGER NOT NULL,
  provider_name TEXT NOT NULL,
  -- ui.v1 CalendarColor
  color INTEGER NOT NULL,
  shown INTEGER NOT NULL
);

CREATE TABLE events (
  id TEXT PRIMARY KEY,
  calendar_id TEXT NOT NULL REFERENCES calendars (id) ON DELETE CASCADE,
  -- "2026-11-11"
  day TEXT NOT NULL,
  start_minute INTEGER NOT NULL,
  end_minute INTEGER NOT NULL,
  title TEXT NOT NULL,
  place TEXT NOT NULL
);

CREATE TABLE event_guests (
  event_id TEXT NOT NULL REFERENCES events (id) ON DELETE CASCADE,
  position INTEGER NOT NULL,
  name TEXT NOT NULL,
  PRIMARY KEY (event_id, position)
);

-- An event someone invited the user to.
CREATE TABLE invitations (
  event_id TEXT PRIMARY KEY REFERENCES events (id) ON DELETE CASCADE,
  -- "Mira"
  invited_by TEXT NOT NULL,
  -- "Mira, you and 2 others"
  people TEXT NOT NULL,
  -- world::Answer
  answer INTEGER NOT NULL
);

CREATE TABLE editions (
  id TEXT PRIMARY KEY,
  written_at INTEGER NOT NULL,
  byline TEXT NOT NULL,
  caught_mail INTEGER NOT NULL,
  calendar_title TEXT NOT NULL,
  unread INTEGER NOT NULL
);

CREATE TABLE edition_lines (
  edition_id TEXT NOT NULL REFERENCES editions (id) ON DELETE CASCADE,
  -- Section
  section INTEGER NOT NULL,
  position INTEGER NOT NULL,
  source TEXT NOT NULL,
  text TEXT NOT NULL,
  PRIMARY KEY (edition_id, section, position)
);

CREATE TABLE edition_waiting (
  edition_id TEXT NOT NULL REFERENCES editions (id) ON DELETE CASCADE,
  position INTEGER NOT NULL,
  -- The approval the item opens; it may be resolved and gone since
  approval_id TEXT NOT NULL,
  title TEXT NOT NULL,
  detail TEXT NOT NULL,
  PRIMARY KEY (edition_id, position)
);

CREATE TABLE edition_events (
  edition_id TEXT NOT NULL REFERENCES editions (id) ON DELETE CASCADE,
  position INTEGER NOT NULL,
  time TEXT NOT NULL,
  title TEXT NOT NULL,
  detail TEXT NOT NULL,
  PRIMARY KEY (edition_id, position)
);

CREATE TABLE edition_news (
  edition_id TEXT NOT NULL REFERENCES editions (id) ON DELETE CASCADE,
  topic TEXT NOT NULL,
  position INTEGER NOT NULL,
  headline TEXT NOT NULL,
  source TEXT NOT NULL,
  PRIMARY KEY (edition_id, position)
);

-- News Bearer's topics, as many as the Backend allows.
CREATE TABLE news_topics (
  position INTEGER PRIMARY KEY,
  name TEXT NOT NULL UNIQUE
);

CREATE TABLE quarantine (
  id TEXT PRIMARY KEY,
  sender TEXT NOT NULL,
  account_id TEXT NOT NULL REFERENCES accounts (id) ON DELETE CASCADE,
  received_at INTEGER NOT NULL,
  subject TEXT NOT NULL,
  reason TEXT NOT NULL,
  caught_by TEXT NOT NULL,
  why TEXT NOT NULL,
  outcome TEXT NOT NULL,
  body TEXT NOT NULL,
  -- When the Backend drops its copy
  drops_at INTEGER NOT NULL
);

-- A rule: one condition and its actions, evaluated in code on every new mail.
CREATE TABLE rules (
  id TEXT PRIMARY KEY,
  position INTEGER NOT NULL,
  -- ConditionKind
  condition INTEGER NOT NULL,
  -- Sender contains and sender domain only: the text, in lower case
  parameter TEXT,
  enabled INTEGER NOT NULL
);

-- Subject contains only: the words, any of which holds.
CREATE TABLE rule_words (
  rule_id TEXT NOT NULL REFERENCES rules (id) ON DELETE CASCADE,
  position INTEGER NOT NULL,
  word TEXT NOT NULL,
  PRIMARY KEY (rule_id, position)
);

CREATE TABLE rule_actions (
  rule_id TEXT NOT NULL REFERENCES rules (id) ON DELETE CASCADE,
  position INTEGER NOT NULL,
  -- ActionKind
  action INTEGER NOT NULL,
  -- Move to folder only
  folder_id TEXT REFERENCES folders (id) ON DELETE CASCADE,
  -- Hand to agent only
  agent_id TEXT REFERENCES agents (id) ON DELETE CASCADE,
  PRIMARY KEY (rule_id, position)
);

CREATE TABLE blocked (
  address TEXT PRIMARY KEY,
  position INTEGER NOT NULL,
  -- Who blocked it: "Security Guard", "you"
  blocked_by TEXT NOT NULL,
  blocked_at INTEGER NOT NULL
);

-- Whether a model key is set. The key itself lives on the Backend only.
CREATE TABLE model_keys (
  -- ui.v1 ModelProvider
  provider INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  is_set INTEGER NOT NULL
);

-- The models the app offers.
CREATE TABLE models (
  name TEXT PRIMARY KEY,
  position INTEGER NOT NULL,
  -- ui.v1 ModelProvider
  provider INTEGER NOT NULL
);

-- The tasks a model is picked for: world::TASK_REPLIES and the others.
CREATE TABLE task_models (
  id TEXT PRIMARY KEY,
  position INTEGER NOT NULL,
  label TEXT NOT NULL,
  hint TEXT NOT NULL,
  model_name TEXT NOT NULL REFERENCES models (name)
);

CREATE TABLE settings (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  notify_when_waiting INTEGER NOT NULL,
  agents_paused INTEGER NOT NULL,
  -- Daily token budget; NULL for none
  budget_tokens INTEGER,
  -- The tone AI compose writes in
  compose_tone TEXT NOT NULL,
  -- world::Ai: whether model work runs, as the Backend says
  ai INTEGER NOT NULL,
  -- How many days Quarantine keeps a copy, as the Backend says
  quarantine_days INTEGER NOT NULL,
  -- How many of the latest messages a dry run reads, as the Backend says
  dry_run_messages INTEGER NOT NULL
);

-- When each snooze the Backend offers ends.
CREATE TABLE snooze_times (
  -- world::Snooze
  preset INTEGER PRIMARY KEY,
  until INTEGER NOT NULL
);

-- The agent being built, kept until it is created or discarded. Its permissions are
-- builder_permissions.
CREATE TABLE builder (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  name TEXT NOT NULL,
  instructions TEXT NOT NULL,
  -- builder::SCHEDULES id
  schedule TEXT NOT NULL,
  -- The custom schedule's interval
  interval_seconds INTEGER NOT NULL,
  ask_before_new_recipient INTEGER NOT NULL
);

-- builder::TRIGGERS ids.
CREATE TABLE builder_triggers (
  trigger_id TEXT PRIMARY KEY,
  chosen INTEGER NOT NULL
);

-- The filter choices picked, as in agent_filters.
CREATE TABLE builder_filters (
  trigger_id TEXT NOT NULL,
  filter_id TEXT NOT NULL,
  choice_id TEXT NOT NULL,
  PRIMARY KEY (trigger_id, filter_id)
);

CREATE TABLE builder_accounts (
  account_id TEXT PRIMARY KEY REFERENCES accounts (id) ON DELETE CASCADE
);
