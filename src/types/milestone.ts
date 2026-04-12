export interface Milestone {
  id: string;
  goal_id: string;
  milestone_order: number;
  parent_milestone_id: string | null;
  plan: string | null;
  plan_summary: string | null;
  skill_path: string | null;
  status: MilestoneStatus;
  result_summary: string | null;
  created_at: string;
  updated_at: string;
}

export type MilestoneStatus =
  | 'planned'
  | 'in_progress'
  | 'completed'
  | 'failed'
  | 'pruned';

export interface Plan {
  steps: PlanStep[];
  dependencies: [number, number][];
}

export interface PlanStep {
  id: string;
  description: string;
  skill_requirements: SkillRequirement[];
  status: PlanStepStatus;
  result: string | null;
}

export type PlanStepStatus = 'pending' | 'in_progress' | 'completed' | 'failed';

export interface SkillRequirement {
  name: string;
  description: string;
  parameters_schema: Record<string, unknown> | null;
  priority: number;
}

export interface CreateMilestoneRequest {
  goal_id: string;
  milestone_order: number;
  parent_milestone_id?: string;
}
