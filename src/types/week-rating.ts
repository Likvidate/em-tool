import type { Color } from "../lib/colors";

export interface WeekRating {
  id: number;
  reportId: number | null;             // null = team-overall
  isoWeek: string;                     // "YYYY-Www"
  color: Color;
  notes: string | null;
  createdAt: number;
  updatedAt: number;
}

export interface UpsertWeekRatingInput {
  reportId: number | null;
  isoWeek: string;
  color: Color;
  notes: string | null;
}
