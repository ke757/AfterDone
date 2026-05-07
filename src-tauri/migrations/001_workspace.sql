-- Initial schema
-- WorkSpace and WorkNode model for workhub mod

PRAGMA foreign_keys = ON;

-- Goals table: each goal is a persistent entity
CREATE TABLE IF NOT EXISTS goals (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    summary TEXT,
    content TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'draft',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Agent logs table: append-only log of agent activity
CREATE TABLE IF NOT EXISTS agent_logs (
    id TEXT PRIMARY KEY NOT NULL,
    goal_id TEXT NOT NULL,
    agent_type TEXT NOT NULL,
    phase TEXT NOT NULL,
    event_type TEXT NOT NULL,
    payload TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (goal_id) REFERENCES goals(id) ON DELETE CASCADE
);

-- WorkSpaces table: 1:1 with Goal
CREATE TABLE IF NOT EXISTS workspaces (
    id TEXT PRIMARY KEY NOT NULL,
    goal_id TEXT NOT NULL UNIQUE,
    current_node_id TEXT,

    -- WorkSpace-level resources
    goal_md TEXT,                       -- GOAL.md content
    plan_md TEXT,                       -- PLAN.md content

    -- Metadata
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),

    FOREIGN KEY (goal_id) REFERENCES goals(id) ON DELETE CASCADE,
    FOREIGN KEY (current_node_id) REFERENCES worknodes(id) ON DELETE SET NULL
);

-- WorkNodes table: version nodes forming a version tree
CREATE TABLE IF NOT EXISTS worknodes (
    id TEXT PRIMARY KEY NOT NULL,
    workspace_id TEXT NOT NULL,
    parent_node_id TEXT,
    node_order INTEGER NOT NULL DEFAULT 0,
    node_type TEXT NOT NULL DEFAULT 'version',   -- 'root' | 'version'

    status TEXT NOT NULL DEFAULT 'planned',

    -- WorkNode-level resources
    bug_md TEXT,                        -- BUG.md content (JSON array of BugEntry)
    user_manual_md TEXT,                -- USER_MANUAL.md content
    conclusion_md TEXT,                 -- CONCLUSION.md content
    -- Metadata
    plan_summary TEXT,
    result_summary TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),

    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE,
    FOREIGN KEY (parent_node_id) REFERENCES worknodes(id) ON DELETE SET NULL
);

-- GeneratorTasks table: tracks HarnessAgent tasks
CREATE TABLE IF NOT EXISTS generator_tasks (
    id TEXT PRIMARY KEY NOT NULL,
    workspace_id TEXT NOT NULL,
    worknode_id TEXT,
    specification TEXT NOT NULL,        -- The spec sent to HarnessAgent
    status TEXT NOT NULL DEFAULT 'pending',  -- pending/running/completed/failed
    result TEXT,                        -- Result from HarnessAgent
    user_manual TEXT,                   -- USER_MANUAL.md from HarnessAgent
    skills_json TEXT,                   -- Skills JSON from HarnessAgent
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),

    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE,
    FOREIGN KEY (worknode_id) REFERENCES worknodes(id) ON DELETE SET NULL
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_goals_status ON goals(status);
CREATE INDEX IF NOT EXISTS idx_agent_logs_goal ON agent_logs(goal_id, created_at);
CREATE INDEX IF NOT EXISTS idx_workspaces_goal ON workspaces(goal_id);
CREATE INDEX IF NOT EXISTS idx_workspaces_current_node ON workspaces(current_node_id);
CREATE INDEX IF NOT EXISTS idx_worknodes_workspace ON worknodes(workspace_id);
CREATE INDEX IF NOT EXISTS idx_worknodes_parent ON worknodes(parent_node_id);
CREATE INDEX IF NOT EXISTS idx_worknodes_status ON worknodes(status);
CREATE INDEX IF NOT EXISTS idx_generator_tasks_workspace ON generator_tasks(workspace_id);
CREATE INDEX IF NOT EXISTS idx_generator_tasks_status ON generator_tasks(status);
