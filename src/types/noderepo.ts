/**
 * NodeSpace and WorkNode types for the NodeRepo module
 */

/**
 * NodeSpace: the workspace for a goal (1:1 with Goal)
 */
export interface NodeSpace {
  id: string;
  goal_id: string;
  current_node_id: string | null;
  goal_md: string | null;
  plan_md: string | null;
  created_at: string;
  updated_at: string;
}

/**
 * WorkNode: a milestone node in the node tree
 */
export interface WorkNode {
  id: string;
  nodespace_id: string;
  parent_node_id: string | null;
  node_order: number;
  status: WorkNodeStatus;
  bug_md: string | null;
  user_manual_md: string | null;
  conclusion_md: string | null;
  milestone_id: string | null;
  plan_summary: string | null;
  result_summary: string | null;
  created_at: string;
  updated_at: string;
}

/**
 * WorkNode status enum
 */
export type WorkNodeStatus = 'planned' | 'running' | 'completed' | 'failed';

/**
 * GeneratorTask: tracks HarnessAgent tasks
 */
export interface GeneratorTask {
  id: string;
  nodespace_id: string;
  worknode_id: string | null;
  specification: string;
  status: GeneratorTaskStatus;
  result: string | null;
  user_manual: string | null;
  skills_json: string | null;
  created_at: string;
  updated_at: string;
}

/**
 * GeneratorTask status
 */
export type GeneratorTaskStatus = 'pending' | 'running' | 'completed' | 'failed';

/**
 * Bug entry stored in BUG.md
 */
export interface BugEntry {
  id: string;
  description: string;
  error_output: string | null;
  created_at: string;
  resolved: boolean;
}

/**
 * Create NodeSpace request
 */
export interface CreateNodeSpaceRequest {
  goal_id: string;
  goal_md?: string;
  plan_md?: string;
}

/**
 * Create WorkNode request
 */
export interface CreateWorkNodeRequest {
  nodespace_id: string;
  parent_node_id?: string;
  node_order: number;
  milestone_id?: string;
}

/**
 * WorkNode tree structure for displaying node hierarchy
 */
export interface WorkNodeTree {
  node: WorkNode;
  children: WorkNodeTree[];
}

/**
 * Module specification for Generator
 */
export interface ModuleSpec {
  name: string;
  description: string;
  skill_requirements: GeneratorSkillRequirement[];
}

/**
 * Skill requirement for Generator (different from Milestone's SkillRequirement)
 */
export interface GeneratorSkillRequirement {
  name: string;
  description: string;
  parameters_schema?: Record<string, unknown>;
}

/**
 * Specification document for HarnessAgent
 */
export interface Specification {
  project_background: string;
  module_breakdown: ModuleSpec[];
  acceptance_criteria: string[];
}
