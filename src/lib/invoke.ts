import { invoke as rawInvoke } from "@tauri-apps/api/core";
import type { Report, NewReportInput, UpdateReportInput } from "../types/report";
import type { WeekRating, UpsertWeekRatingInput } from "../types/week-rating";
import type { OneOnOne, NewOneOnOneInput, UpdateOneOnOneInput } from "../types/one-on-one";
import type { ActionItem, NewActionItemInput } from "../types/action-item";
import type { PerformanceReview, NewPerformanceReviewInput } from "../types/performance-review";
import type { GeneratedPlan, GeneratePlanInput } from "../types/generated-plan";

export type CommandError = { code: string; message: string };

export class InvokeError extends Error {
  code: string;
  constructor(err: CommandError) {
    super(err.message);
    this.code = err.code;
  }
}

export async function invoke<T = void>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  try {
    return await rawInvoke<T>(command, args);
  } catch (err) {
    if (err && typeof err === "object" && "code" in err && "message" in err) {
      throw new InvokeError(err as CommandError);
    }
    throw err;
  }
}

export const vaultApi = {
  exists: () => invoke<boolean>("vault_exists"),
  isUnlocked: () => invoke<boolean>("is_unlocked"),
  create: (password: string) => invoke<void>("create_vault", { password }),
  unlock: (password: string) => invoke<void>("unlock_vault", { password }),
  lock: () => invoke<void>("lock_vault"),
  touchActivity: () => invoke<void>("touch_activity"),
};

export const reportsApi = {
  list: (includeArchived = false) =>
    invoke<Report[]>("list_reports", { includeArchived }),
  get: (id: number) => invoke<Report | null>("get_report", { id }),
  create: (input: NewReportInput) => invoke<Report>("create_report", { input }),
  update: (input: UpdateReportInput) => invoke<Report>("update_report", { input }),
  archive: (id: number) => invoke<void>("archive_report", { id }),
  delete: (id: number) => invoke<void>("delete_report", { id }),
};

export const weekRatingsApi = {
  listByWeek: (isoWeek: string) =>
    invoke<WeekRating[]>("list_week_ratings_by_week", { isoWeek }),
  listByReport: (reportId: number) =>
    invoke<WeekRating[]>("list_week_ratings_by_report", { reportId }),
  listTeamOverall: () =>
    invoke<WeekRating[]>("list_week_ratings_team_overall"),
  listInRange: (fromIsoWeek: string, toIsoWeek: string) =>
    invoke<WeekRating[]>("list_week_ratings_in_range", { fromIsoWeek, toIsoWeek }),
  upsert: (input: UpsertWeekRatingInput) =>
    invoke<WeekRating>("upsert_week_rating", { input }),
  delete: (reportId: number | null, isoWeek: string) =>
    invoke<void>("delete_week_rating", { reportId, isoWeek }),
};

export const oneOnOnesApi = {
  list: (reportId: number) => invoke<OneOnOne[]>("list_one_on_ones", { reportId }),
  create: (input: NewOneOnOneInput) => invoke<OneOnOne>("create_one_on_one", { input }),
  update: (input: UpdateOneOnOneInput) => invoke<OneOnOne>("update_one_on_one", { input }),
  delete: (id: number) => invoke<void>("delete_one_on_one", { id }),
};

export const actionItemsApi = {
  listByMeeting: (oneOnOneId: number) => invoke<ActionItem[]>("list_action_items_by_meeting", { oneOnOneId }),
  listByReport: (reportId: number) => invoke<ActionItem[]>("list_action_items_by_report", { reportId }),
  listOpen: (reportId: number) => invoke<ActionItem[]>("list_open_action_items", { reportId }),
  create: (input: NewActionItemInput) => invoke<ActionItem>("create_action_item", { input }),
  toggle: (id: number) => invoke<ActionItem>("toggle_action_item", { id }),
  delete: (id: number) => invoke<void>("delete_action_item", { id }),
};

export const reviewsApi = {
  list: (reportId: number) => invoke<PerformanceReview[]>("list_reviews", { reportId }),
  create: (input: NewPerformanceReviewInput) => invoke<PerformanceReview>("create_review", { input }),
  delete: (id: number) => invoke<void>("delete_review", { id }),
};

export const plansApi = {
  list: (reportId: number) => invoke<GeneratedPlan[]>("list_generated_plans", { reportId }),
  generateTemplate: (input: GeneratePlanInput) => invoke<GeneratedPlan>("generate_plan_template", { input }),
  generateClaude: (input: GeneratePlanInput) => invoke<GeneratedPlan>("generate_plan_claude", { input }),
  attachToMeeting: (planId: number, oneOnOneId: number) =>
    invoke<void>("attach_plan_to_meeting", { planId, oneOnOneId }),
};

export const settingsApi = {
  hasApiKey: () => invoke<boolean>("get_api_key_set"),
  setApiKey: (value: string | null) => invoke<void>("set_api_key", { value }),
};
