import { defineStore } from "pinia";
import { ref } from "vue";
import { reviewsApi, InvokeError } from "../lib/invoke";
import type { PerformanceReview, NewPerformanceReviewInput } from "../types/performance-review";

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

  function forReport(reportId: number): PerformanceReview[] {
    return byReport.value[reportId] ?? [];
  }

  return { byReport, loading, lastError, loadForReport, create, forReport };
});
