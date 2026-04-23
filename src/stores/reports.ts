import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { reportsApi, InvokeError } from "../lib/invoke";
import type { Report, NewReportInput, UpdateReportInput } from "../types/report";

export const useReportsStore = defineStore("reports", () => {
  const items = ref<Report[]>([]);
  const loaded = ref(false);
  const loading = ref(false);
  const lastError = ref<string | null>(null);

  const active = computed(() => items.value.filter((r) => r.active));

  async function load(includeArchived = false) {
    loading.value = true;
    lastError.value = null;
    try {
      items.value = await reportsApi.list(includeArchived);
      loaded.value = true;
    } catch (e) {
      lastError.value = e instanceof InvokeError ? e.message : String(e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function create(input: NewReportInput): Promise<Report> {
    const created = await reportsApi.create(input);
    items.value = [...items.value, created].sort((a, b) =>
      a.name.localeCompare(b.name, undefined, { sensitivity: "base" }),
    );
    return created;
  }

  async function update(input: UpdateReportInput): Promise<Report> {
    const updated = await reportsApi.update(input);
    items.value = items.value.map((r) => (r.id === updated.id ? updated : r));
    return updated;
  }

  async function archive(id: number) {
    await reportsApi.archive(id);
    items.value = items.value.map((r) => (r.id === id ? { ...r, active: false } : r));
  }

  async function remove(id: number) {
    await reportsApi.delete(id);
    items.value = items.value.filter((r) => r.id !== id);
  }

  function byId(id: number): Report | undefined {
    return items.value.find((r) => r.id === id);
  }

  return { items, active, loaded, loading, lastError, load, create, update, archive, remove, byId };
});
