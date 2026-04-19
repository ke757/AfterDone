-- WorkSpace and WorkNode schema for workhub mod
-- Extends the existing goal/milestone model with resource management

PRAGMA foreign_keys = ON;

-- WorkSpaces table: extends goals with resource management
-- Each WorkSpace corresponds 1:1 with a Goal
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

-- WorkNodes table: extends milestones with resource management
-- Replaces milestones for workhub-managed goals
CREATE TABLE IF NOT EXISTS worknodes (
    id TEXT PRIMARY KEY NOT NULL,
    workspace_id TEXT NOT NULL,
    parent_node_id TEXT,
    node_order INTEGER NOT NULL DEFAULT 0,
    
    -- Status: planned/running/completed/failed
    status TEXT NOT NULL DEFAULT 'planned',
    
    -- WorkNode-level resources
    bug_md TEXT,                        -- BUG.md content (JSON array of BugEntry)
    user_manual_md TEXT,                -- USER_MANUAL.md content
    conclusion_md TEXT,                 -- CONCLUSION.md content
    
    -- Link to milestone for skills association
    milestone_id TEXT,
    
    -- Metadata
    plan_summary TEXT,
    result_summary TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE,
    FOREIGN KEY (parent_node_id) REFERENCES worknodes(id) ON DELETE SET NULL,
    FOREIGN KEY (milestone_id) REFERENCES milestones(id) ON DELETE SET NULL
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
CREATE INDEX IF NOT EXISTS idx_workspaces_goal ON workspaces(goal_id);
CREATE INDEX IF NOT EXISTS idx_workspaces_current_node ON workspaces(current_node_id);
CREATE INDEX IF NOT EXISTS idx_worknodes_workspace ON worknodes(workspace_id);
CREATE INDEX IF NOT EXISTS idx_worknodes_parent ON worknodes(parent_node_id);
CREATE INDEX IF NOT EXISTS idx_worknodes_status ON worknodes(status);
CREATE INDEX IF NOT EXISTS idx_worknodes_milestone ON worknodes(milestone_id);
CREATE INDEX IF NOT EXISTS idx_generator_tasks_workspace ON generator_tasks(workspace_id);
CREATE INDEX IF NOT EXISTS idx_generator_tasks_status ON generator_tasks(status);
