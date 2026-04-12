-- NodeSpace and WorkNode schema for RigYourGoal
-- Extends the existing goal/milestone model with resource management

PRAGMA foreign_keys = ON;

-- NodeSpaces table: extends goals with resource management
-- Each NodeSpace corresponds 1:1 with a Goal
CREATE TABLE IF NOT EXISTS nodespaces (
    id TEXT PRIMARY KEY NOT NULL,
    goal_id TEXT NOT NULL UNIQUE,
    current_node_id TEXT,
    
    -- NodeSpace-level resources
    goal_md TEXT,                       -- GOAL.md content
    plan_md TEXT,                       -- PLAN.md content
    
    -- Metadata
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    
    FOREIGN KEY (goal_id) REFERENCES goals(id) ON DELETE CASCADE,
    FOREIGN KEY (current_node_id) REFERENCES worknodes(id) ON DELETE SET NULL
);

-- WorkNodes table: extends milestones with resource management
-- Replaces milestones for noderepo-managed goals
CREATE TABLE IF NOT EXISTS worknodes (
    id TEXT PRIMARY KEY NOT NULL,
    nodespace_id TEXT NOT NULL,
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
    
    FOREIGN KEY (nodespace_id) REFERENCES nodespaces(id) ON DELETE CASCADE,
    FOREIGN KEY (parent_node_id) REFERENCES worknodes(id) ON DELETE SET NULL,
    FOREIGN KEY (milestone_id) REFERENCES milestones(id) ON DELETE SET NULL
);

-- GeneratorTasks table: tracks HarnessAgent tasks
CREATE TABLE IF NOT EXISTS generator_tasks (
    id TEXT PRIMARY KEY NOT NULL,
    nodespace_id TEXT NOT NULL,
    worknode_id TEXT,
    specification TEXT NOT NULL,        -- The spec sent to HarnessAgent
    status TEXT NOT NULL DEFAULT 'pending',  -- pending/running/completed/failed
    result TEXT,                        -- Result from HarnessAgent
    user_manual TEXT,                   -- USER_MANUAL.md from HarnessAgent
    skills_json TEXT,                   -- Skills JSON from HarnessAgent
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    
    FOREIGN KEY (nodespace_id) REFERENCES nodespaces(id) ON DELETE CASCADE,
    FOREIGN KEY (worknode_id) REFERENCES worknodes(id) ON DELETE SET NULL
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_nodespaces_goal ON nodespaces(goal_id);
CREATE INDEX IF NOT EXISTS idx_nodespaces_current_node ON nodespaces(current_node_id);
CREATE INDEX IF NOT EXISTS idx_worknodes_nodespace ON worknodes(nodespace_id);
CREATE INDEX IF NOT EXISTS idx_worknodes_parent ON worknodes(parent_node_id);
CREATE INDEX IF NOT EXISTS idx_worknodes_status ON worknodes(status);
CREATE INDEX IF NOT EXISTS idx_worknodes_milestone ON worknodes(milestone_id);
CREATE INDEX IF NOT EXISTS idx_generator_tasks_nodespace ON generator_tasks(nodespace_id);
CREATE INDEX IF NOT EXISTS idx_generator_tasks_status ON generator_tasks(status);
