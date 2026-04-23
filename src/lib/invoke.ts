import { invoke as rawInvoke } from "@tauri-apps/api/core";
import type { Report, NewReportInput, UpdateReportInput } from "../types/report";
import type { WeekRating, UpsertWeekRatingInput } from "../types/week-rating";

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
