import { defineStore } from "pinia";
import { ref } from "vue";
import { plansApi, InvokeError } from "../lib/invoke";
import type { GeneratedPlan, GeneratePlanInput } from "../types/generated-plan";

export const useGeneratedPlansStore = defineStore("generatedPlans", () => {
  const byReport = ref<Record<number, GeneratedPlan[]>>({});
  const generating = ref(false);
  const lastError = ref<string | null>(null);

  async function loadForReport(reportId: number) {
    byReport.value = { ...byReport.value, [reportId]: await plansApi.list(reportId) };
  }

  async function generate(input: GeneratePlanInput): Promise<GeneratedPlan> {
    generating.value = true;
    lastError.value = null;
    try {
      let fn: (input: GeneratePlanInput) => Promise<GeneratedPlan>;
      if (input.source === "claude") fn = plansApi.generateClaude;
      else if (input.source === "ollama") fn = plansApi.generateOllama;
      else fn = plansApi.generateTemplate;
      const plan = await fn(input);
      const cur = byReport.value[input.targetReportId] ?? [];
      byReport.value = { ...byReport.value, [input.targetReportId]: [plan, ...cur] };
      return plan;
    } catch (e) {
      lastError.value = e instanceof InvokeError ? e.message : String(e);
      throw e;
    } finally {
      generating.value = false;
    }
  }

  async function attachToMeeting(planId: number, oneOnOneId: number, reportId: number) {
    await plansApi.attachToMeeting(planId, oneOnOneId);
    byReport.value = {
      ...byReport.value,
      [reportId]: (byReport.value[reportId] ?? []).map((p) =>
        p.id === planId ? { ...p, savedToMeetingId: oneOnOneId } : p,
      ),
    };
  }

  function forReport(reportId: number): GeneratedPlan[] {
    return byReport.value[reportId] ?? [];
  }

  return { byReport, generating, lastError, loadForReport, generate, attachToMeeting, forReport };
});
