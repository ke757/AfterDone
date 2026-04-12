export interface Skill {
  id: string;
  milestone_id: string;
  name: string;
  description: string;
  harness_skill_id: string | null;
  parameters_schema: string | null;
  test_report: string | null;
  status: SkillStatus;
  source_code_ref: string | null;
  created_at: string;
  updated_at: string;
}

export type SkillStatus = 'requested' | 'generated' | 'verified' | 'failed';

export interface SkillCommand {
  id: string;
  name: string;
  description: string;
  icon: string | null;
}

export interface CreateSkillRequest {
  milestone_id: string;
  name: string;
  description: string;
}
