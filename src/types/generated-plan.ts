export type PlanKind = "one_on_one" | "review";
export type PlanSource = "claude" | "template" | "ollama";

export type WindowSpec =
  | { type: "since_last_one_on_one" }
  | { type: "last_n_weeks"; n: number }
  | { type: "since_last_review" }
  | { type: "custom"; from: string; to: string };

export interface GeneratedPlan {
  id: number;
  kind: PlanKind;
  targetReportId: number;
  windowSpec: string;        // JSON string of WindowSpec
  source: PlanSource;
  promptMd: string | null;
  outputMd: string;
  savedToMeetingId: number | null;
  createdAt: number;
}

export interface GeneratePlanInput {
  kind: PlanKind;
  targetReportId: number;
  windowSpec: WindowSpec;
  source: PlanSource;
}
