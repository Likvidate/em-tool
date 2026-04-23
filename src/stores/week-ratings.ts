import { defineStore } from "pinia";
import { ref } from "vue";
import { weekRatingsApi, InvokeError } from "../lib/invoke";
import type { WeekRating, UpsertWeekRatingInput } from "../types/week-rating";

/**
 * Keyed cache of ratings for fast access from multiple views.
 * Key format: `${reportId ?? "team"}:${isoWeek}` — unique per rating.
 */
function keyOf(reportId: number | null, isoWeek: string): string {
  return `${reportId ?? "team"}:${isoWeek}`;
}

export const useWeekRatingsStore = defineStore("weekRatings", () => {
  const byKey = ref<Record<string, WeekRating>>({});
  const loading = ref(false);
  const lastError = ref<string | null>(null);

  function indexMany(rows: WeekRating[]) {
    const next = { ...byKey.value };
    for (const r of rows) {
      next[keyOf(r.reportId, r.isoWeek)] = r;
    }
    byKey.value = next;
  }

  async function loadWeek(isoWeek: string) {
    loading.value = true;
    lastError.value = null;
    try {
      indexMany(await weekRatingsApi.listByWeek(isoWeek));
    } catch (e) {
      lastError.value = e instanceof InvokeError ? e.message : String(e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function loadForReport(reportId: number) {
    loading.value = true;
    lastError.value = null;
    try {
      indexMany(await weekRatingsApi.listByReport(reportId));
    } catch (e) {
      lastError.value = e instanceof InvokeError ? e.message : String(e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function loadRange(fromIsoWeek: string, toIsoWeek: string) {
    loading.value = true;
    lastError.value = null;
    try {
      indexMany(await weekRatingsApi.listInRange(fromIsoWeek, toIsoWeek));
      indexMany(await weekRatingsApi.listTeamOverall());
    } catch (e) {
      lastError.value = e instanceof InvokeError ? e.message : String(e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function upsert(input: UpsertWeekRatingInput): Promise<WeekRating> {
    const saved = await weekRatingsApi.upsert(input);
    byKey.value = { ...byKey.value, [keyOf(saved.reportId, saved.isoWeek)]: saved };
    return saved;
  }

  async function remove(reportId: number | null, isoWeek: string) {
    await weekRatingsApi.delete(reportId, isoWeek);
    const next = { ...byKey.value };
    delete next[keyOf(reportId, isoWeek)];
    byKey.value = next;
  }

  function get(reportId: number | null, isoWeek: string): WeekRating | undefined {
    return byKey.value[keyOf(reportId, isoWeek)];
  }

  return { byKey, loading, lastError, loadWeek, loadForReport, loadRange, upsert, remove, get };
});
