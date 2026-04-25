export interface Goal {
  id: string;
  title: string;
  summary: string | null;
  raw_input: string;
  status: GoalStatus;
  parent_goal_id: string | null;
  fork_context: string | null;
  current_milestone_id: string | null;
  created_at: string;
  updated_at: string;
}

export type GoalStatus =
  | 'draft'
  | 'pinned'
  | 'building'
  | 'reached'
  | 'optimizing'
  | 'archived'
  | 'failed';

export interface GoalSummary {
  title: string;
  description: string;
  acceptance_criteria: string[];
  constraints: string[];
  refinement_questions: string[];
}

export interface CreateGoalInput {
  raw_input: string;
}

export type CreateGoalRequest = CreateGoalInput;

export interface UpdateGoalInput {
  title?: string;
  summary?: string;
  raw_input?: string;
}

export interface GoalTree {
  goal: Goal;
  children: GoalTree[];
}
