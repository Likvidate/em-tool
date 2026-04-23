import { defineStore } from "pinia";
import { ref } from "vue";
import { oneOnOnesApi, InvokeError } from "../lib/invoke";
import type { OneOnOne, NewOneOnOneInput } from "../types/one-on-one";

export const useOneOnOnesStore = defineStore("oneOnOnes", () => {
  const byReport = ref<Record<number, OneOnOne[]>>({});
  const loading = ref(false);
  const lastError = ref<string | null>(null);

  async function loadForReport(reportId: number) {
    loading.value = true;
    lastError.value = null;
    try {
      byReport.value = { ...byReport.value, [reportId]: await oneOnOnesApi.list(reportId) };
    } catch (e) {
      lastError.value = e instanceof InvokeError ? e.message : String(e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function create(input: NewOneOnOneInput): Promise<OneOnOne> {
    const created = await oneOnOnesApi.create(input);
    const cur = byReport.value[input.reportId] ?? [];
    byReport.value = {
      ...byReport.value,
      [input.reportId]: [created, ...cur].sort((a, b) => b.occurredAt - a.occurredAt),
    };
    return created;
  }

  async function remove(id: number, reportId: number) {
    await oneOnOnesApi.delete(id);
    byReport.value = {
      ...byReport.value,
      [reportId]: (byReport.value[reportId] ?? []).filter((m) => m.id !== id),
    };
  }

  function forReport(reportId: number): OneOnOne[] {
    return byReport.value[reportId] ?? [];
  }

  return { byReport, loading, lastError, loadForReport, create, remove, forReport };
});
