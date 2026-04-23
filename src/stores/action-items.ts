import { defineStore } from "pinia";
import { ref } from "vue";
import { actionItemsApi, InvokeError } from "../lib/invoke";
import type { ActionItem, NewActionItemInput } from "../types/action-item";

export const useActionItemsStore = defineStore("actionItems", () => {
  const byReport = ref<Record<number, ActionItem[]>>({});
  const loading = ref(false);
  const lastError = ref<string | null>(null);

  async function loadForReport(reportId: number) {
    loading.value = true;
    try {
      byReport.value = { ...byReport.value, [reportId]: await actionItemsApi.listByReport(reportId) };
    } catch (e) {
      lastError.value = e instanceof InvokeError ? e.message : String(e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function create(input: NewActionItemInput): Promise<ActionItem> {
    const created = await actionItemsApi.create(input);
    const cur = byReport.value[input.reportId] ?? [];
    byReport.value = { ...byReport.value, [input.reportId]: [created, ...cur] };
    return created;
  }

  async function toggle(id: number, reportId: number) {
    const updated = await actionItemsApi.toggle(id);
    byReport.value = {
      ...byReport.value,
      [reportId]: (byReport.value[reportId] ?? []).map((a) => (a.id === id ? updated : a)),
    };
  }

  async function remove(id: number, reportId: number) {
    await actionItemsApi.delete(id);
    byReport.value = {
      ...byReport.value,
      [reportId]: (byReport.value[reportId] ?? []).filter((a) => a.id !== id),
    };
  }

  function forReport(reportId: number): ActionItem[] {
    return byReport.value[reportId] ?? [];
  }

  return { byReport, loading, lastError, loadForReport, create, toggle, remove, forReport };
});
