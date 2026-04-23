import { defineStore } from "pinia";
import { ref } from "vue";
import { reviewsApi, InvokeError } from "../lib/invoke";
import type {
  PerformanceReview,
  NewPerformanceReviewInput,
  UpdatePerformanceReviewInput,
} from "../types/performance-review";

export const useReviewsStore = defineStore("reviews", () => {
  const byReport = ref<Record<number, PerformanceReview[]>>({});
  const loading = ref(false);
  const lastError = ref<string | null>(null);

  async function loadForReport(reportId: number) {
    loading.value = true;
    try {
      byReport.value = { ...byReport.value, [reportId]: await reviewsApi.list(reportId) };
    } catch (e) {
      lastError.value = e instanceof InvokeError ? e.message : String(e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function create(input: NewPerformanceReviewInput): Promise<PerformanceReview> {
    const created = await reviewsApi.create(input);
    const cur = byReport.value[input.reportId] ?? [];
    byReport.value = {
      ...byReport.value,
      [input.reportId]: [created, ...cur].sort((a, b) => b.occurredAt - a.occurredAt),
    };
    return created;
  }

  async function update(input: UpdatePerformanceReviewInput): Promise<PerformanceReview> {
    const updated = await reviewsApi.update(input);
    for (const [rid, list] of Object.entries(byReport.value)) {
      const idx = list.findIndex((r) => r.id === updated.id);
      if (idx !== -1) {
        const next = [...list];
        next[idx] = updated;
        next.sort((a, b) => b.occurredAt - a.occurredAt);
        byReport.value = { ...byReport.value, [Number(rid)]: next };
        break;
      }
    }
    return updated;
  }

  function forReport(reportId: number): PerformanceReview[] {
    return byReport.value[reportId] ?? [];
  }

  return { byReport, loading, lastError, loadForReport, create, update, forReport };
});
