<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useReportsStore } from "../stores/reports";
import { useGeneratedPlansStore } from "../stores/generated-plans";
import { useOneOnOnesStore } from "../stores/one-on-ones";
import { settingsApi } from "../lib/invoke";
import type { PlanKind, WindowSpec, GeneratedPlan } from "../types/generated-plan";

const route = useRoute();
const router = useRouter();
const reports = useReportsStore();
const plans = useGeneratedPlansStore();
const oneOnOnes = useOneOnOnesStore();

const selectedReportId = ref<number | null>(null);
const kind = ref<PlanKind>("one_on_one");
const windowChoice = ref<
  | "since_last_one_on_one"
  | "last_4"
  | "last_8"
  | "last_12"
  | "since_last_review"
  | "custom"
>("since_last_one_on_one");
const customFrom = ref("");
const customTo = ref("");
const hasApiKey = ref(false);
const currentPlan = ref<GeneratedPlan | null>(null);
const error = ref<string | null>(null);
const showAttachMenu = ref(false);
const copied = ref(false);

const historyPlans = computed(() =>
  selectedReportId.value ? plans.forReport(selectedReportId.value) : [],
);

const recentMeetings = computed(() =>
  selectedReportId.value ? oneOnOnes.forReport(selectedReportId.value).slice(0, 10) : [],
);

function resolveWindow(): WindowSpec {
  switch (windowChoice.value) {
    case "since_last_one_on_one": return { type: "since_last_one_on_one" };
    case "last_4": return { type: "last_n_weeks", n: 4 };
    case "last_8": return { type: "last_n_weeks", n: 8 };
    case "last_12": return { type: "last_n_weeks", n: 12 };
    case "since_last_review": return { type: "since_last_review" };
    case "custom": return { type: "custom", from: customFrom.value, to: customTo.value };
  }
}

async function doGenerate(source: "claude" | "template") {
  if (!selectedReportId.value) return;
  error.value = null;
  try {
    const plan = await plans.generate({
      kind: kind.value,
      targetReportId: selectedReportId.value,
      windowSpec: resolveWindow(),
      source,
    });
    currentPlan.value = plan;
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : String(e);
  }
}

async function copyOutput() {
  if (!currentPlan.value) return;
  await navigator.clipboard.writeText(currentPlan.value.outputMd);
  copied.value = true;
  setTimeout(() => { copied.value = false; }, 1500);
}

async function attachTo(oneOnOneId: number) {
  if (!currentPlan.value || !selectedReportId.value) return;
  await plans.attachToMeeting(currentPlan.value.id, oneOnOneId, selectedReportId.value);
  currentPlan.value = { ...currentPlan.value, savedToMeetingId: oneOnOneId };
  showAttachMenu.value = false;
}

function formatTs(unix: number): string {
  return new Date(unix * 1000).toISOString().slice(0, 16).replace("T", " ");
}

function loadHistoryPlan(plan: GeneratedPlan) {
  currentPlan.value = plan;
}

onMounted(async () => {
  if (!reports.loaded) await reports.load(false);
  hasApiKey.value = await settingsApi.hasApiKey();
  const routeId = route.params.reportId ? Number(route.params.reportId) : null;
  if (routeId && !Number.isNaN(routeId)) {
    selectedReportId.value = routeId;
  } else if (reports.active.length > 0) {
    selectedReportId.value = reports.active[0].id;
  }
});

watch(selectedReportId, async (id) => {
  if (!id) return;
  currentPlan.value = null;
  await Promise.all([plans.loadForReport(id), oneOnOnes.loadForReport(id)]);
  router.replace({ name: "plan", params: { reportId: String(id) } });
});
</script>

<template>
  <div class="plan-gen">
    <header class="page-header">
      <div>
        <h2 class="page-title">Plan generator</h2>
        <p class="page-subtitle">Draft a 1:1 prep or review prep from recent signals.</p>
      </div>
    </header>

    <section class="card controls card-body">
      <div class="row">
        <label class="field">
          <span class="field-label">Team member</span>
          <select
            v-model="selectedReportId"
            class="field-input"
            :disabled="reports.active.length === 0"
          >
            <option v-if="reports.active.length === 0" :value="null">
              No active reports
            </option>
            <option
              v-for="r in reports.active"
              :key="r.id"
              :value="r.id"
            >
              {{ r.name }}
            </option>
          </select>
        </label>

        <div class="field">
          <span class="field-label">Plan kind</span>
          <div class="toggle-group">
            <button
              type="button"
              class="toggle"
              :class="{ active: kind === 'one_on_one' }"
              @click="kind = 'one_on_one'"
            >
              1:1 prep
            </button>
            <button
              type="button"
              class="toggle"
              :class="{ active: kind === 'review' }"
              @click="kind = 'review'"
            >
              Review prep
            </button>
          </div>
        </div>
      </div>

      <div class="field">
        <span class="field-label">Window</span>
        <div class="pills">
          <button
            type="button"
            class="pill"
            :class="{ active: windowChoice === 'since_last_one_on_one' }"
            @click="windowChoice = 'since_last_one_on_one'"
          >Since last 1:1</button>
          <button
            type="button"
            class="pill"
            :class="{ active: windowChoice === 'last_4' }"
            @click="windowChoice = 'last_4'"
          >Last 4 weeks</button>
          <button
            type="button"
            class="pill"
            :class="{ active: windowChoice === 'last_8' }"
            @click="windowChoice = 'last_8'"
          >Last 8 weeks</button>
          <button
            type="button"
            class="pill"
            :class="{ active: windowChoice === 'last_12' }"
            @click="windowChoice = 'last_12'"
          >Last 12 weeks</button>
          <button
            type="button"
            class="pill"
            :class="{ active: windowChoice === 'since_last_review' }"
            @click="windowChoice = 'since_last_review'"
          >Since last review</button>
          <button
            type="button"
            class="pill"
            :class="{ active: windowChoice === 'custom' }"
            @click="windowChoice = 'custom'"
          >Custom range…</button>
        </div>

        <div v-if="windowChoice === 'custom'" class="custom-range">
          <label class="field">
            <span class="field-label">From</span>
            <input v-model="customFrom" class="field-input mono" placeholder="2025-W01" />
          </label>
          <label class="field">
            <span class="field-label">To</span>
            <input v-model="customTo" class="field-input mono" placeholder="2025-W12" />
          </label>
        </div>
      </div>

      <div class="generate-row">
        <button
          type="button"
          class="btn btn-primary"
          :disabled="plans.generating || !selectedReportId"
          @click="doGenerate('template')"
        >
          <span v-if="plans.generating" class="spinner"></span>
          {{ plans.generating ? "Generating…" : "Generate plan" }}
        </button>
        <button
          v-if="hasApiKey"
          type="button"
          class="btn btn-secondary"
          :disabled="plans.generating || !selectedReportId"
          @click="doGenerate('claude')"
        >
          <span v-if="plans.generating" class="spinner"></span>
          {{ plans.generating ? "Generating…" : "✨ Generate with Claude" }}
        </button>
      </div>

      <div v-if="error" class="error-banner">{{ error }}</div>
    </section>

    <section class="card output-card">
      <div v-if="!currentPlan" class="empty-state">
        Pick options above and generate a plan.
      </div>
      <template v-else>
        <header class="output-head">
          <div class="output-meta">
            <span class="badge" :class="currentPlan.source">
              {{ currentPlan.source === "claude" ? "✨ Claude" : "Template" }}
            </span>
            <span class="kind-tag">
              {{ currentPlan.kind === "one_on_one" ? "1:1 prep" : "Review prep" }}
            </span>
            <span class="ts">{{ formatTs(currentPlan.createdAt) }}</span>
            <span v-if="currentPlan.savedToMeetingId" class="saved">
              saved to 1:1 #{{ currentPlan.savedToMeetingId }}
            </span>
          </div>
          <div class="output-actions">
            <button type="button" class="btn btn-secondary btn-sm" @click="copyOutput">
              {{ copied ? "Copied!" : "Copy markdown" }}
            </button>
            <div class="attach-wrap">
              <button
                type="button"
                class="btn btn-secondary btn-sm"
                :disabled="recentMeetings.length === 0"
                @click="showAttachMenu = !showAttachMenu"
              >
                Save to 1:1 ▾
              </button>
              <div v-if="showAttachMenu" class="attach-menu">
                <div v-if="recentMeetings.length === 0" class="empty-menu">
                  No 1:1s yet.
                </div>
                <button
                  v-for="m in recentMeetings"
                  :key="m.id"
                  type="button"
                  class="attach-item"
                  @click="attachTo(m.id)"
                >
                  <span class="attach-date">{{ formatTs(m.occurredAt) }}</span>
                  <span class="attach-preview">
                    {{ m.agendaMd ? m.agendaMd.slice(0, 60) : "(no agenda)" }}
                  </span>
                </button>
              </div>
            </div>
          </div>
        </header>
        <pre class="output">{{ currentPlan.outputMd }}</pre>
      </template>
    </section>

    <section class="card history card-body">
      <h3>History</h3>
      <div v-if="historyPlans.length === 0" class="empty-history">
        No plans yet for this person.
      </div>
      <ul v-else class="history-list">
        <li
          v-for="p in historyPlans"
          :key="p.id"
          class="history-item"
          :class="{ active: currentPlan?.id === p.id }"
          @click="loadHistoryPlan(p)"
        >
          <span class="h-kind">{{ p.kind === "one_on_one" ? "1:1" : "Review" }}</span>
          <span class="h-sep">·</span>
          <span class="h-source" :class="p.source">
            {{ p.source === "claude" ? "✨ Claude" : "Template" }}
          </span>
          <span class="h-sep">·</span>
          <span class="h-ts">{{ formatTs(p.createdAt) }}</span>
          <span v-if="p.savedToMeetingId" class="h-saved">· saved</span>
        </li>
      </ul>
    </section>
  </div>
</template>

<style scoped>
.plan-gen {
  max-width: 1000px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: var(--space-5);
}

.controls { display: flex; flex-direction: column; gap: var(--space-4); }
.row { display: flex; gap: var(--space-4); flex-wrap: wrap; }

.field { flex: 1; min-width: 220px; }

.mono { font-family: var(--font-mono); }

.toggle-group { display: inline-flex; gap: 0; }
.toggle {
  background: var(--bg-2);
  color: var(--text);
  border: 1px solid var(--border);
  padding: 8px 14px;
  font-size: var(--fs-base);
  cursor: pointer;
  font-family: inherit;
  transition: background var(--t-fast), border-color var(--t-fast), color var(--t-fast);
}
.toggle:first-child { border-radius: var(--radius-md) 0 0 var(--radius-md); }
.toggle:last-child { border-radius: 0 var(--radius-md) var(--radius-md) 0; border-left: none; }
.toggle.active {
  background: var(--accent);
  color: #fff;
  border-color: var(--accent);
}
.toggle:hover:not(.active) {
  border-color: var(--border-strong);
  background: var(--surface-2);
}

.pills { display: inline-flex; gap: 6px; flex-wrap: wrap; }
.pill {
  background: transparent;
  color: var(--text-dim);
  border: 1px solid transparent;
  border-radius: 999px;
  padding: 5px 12px;
  font-size: var(--fs-sm);
  cursor: pointer;
  font-family: inherit;
  transition: background var(--t-fast), border-color var(--t-fast), color var(--t-fast);
}
.pill:hover:not(.active) {
  color: var(--text);
  background: var(--surface-2);
}
.pill.active {
  background: var(--accent-dim);
  color: var(--accent-strong);
  border-color: var(--border-accent);
}

.custom-range { display: flex; gap: var(--space-3); margin-top: var(--space-2); }
.custom-range .field { min-width: 140px; max-width: 180px; }

.generate-row { display: flex; gap: var(--space-2); align-items: flex-start; flex-wrap: wrap; }

.spinner {
  width: 12px; height: 12px;
  border: 2px solid rgba(255, 255, 255, 0.25);
  border-top-color: #fff;
  border-radius: 50%;
  animation: spin 700ms linear infinite;
}
@keyframes spin { to { transform: rotate(360deg); } }

.error-banner {
  padding: 8px 12px;
  background: rgba(239, 68, 68, 0.08);
  border: 1px solid #5a2a2a;
  color: #e5a8a8;
  border-radius: var(--radius-md);
  font-size: var(--fs-sm);
}

.output-card { padding: 0; }
.output-head {
  display: flex; justify-content: space-between; align-items: flex-start;
  gap: var(--space-3); flex-wrap: wrap;
  padding: var(--space-3) var(--space-5);
  border-bottom: 1px solid var(--border);
}
.output-meta { display: flex; gap: var(--space-3); align-items: center; flex-wrap: wrap; }

.badge.claude {
  background: var(--accent-dim);
  color: var(--accent-strong);
  border-color: var(--border-accent);
}
.badge.template {
  background: var(--surface-2);
  color: var(--text-dim);
  border-color: var(--border);
}
.kind-tag {
  font-size: var(--fs-sm);
  color: var(--text-dim);
}
.ts {
  font-family: var(--font-mono);
  font-size: var(--fs-xs);
  color: var(--text-mute);
}
.saved {
  font-size: var(--fs-xs);
  color: var(--success);
}
.output-actions { display: flex; gap: var(--space-2); position: relative; }
.attach-wrap { position: relative; }
.attach-menu {
  position: absolute;
  top: calc(100% + 4px);
  right: 0;
  min-width: 280px;
  max-height: 300px;
  overflow-y: auto;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-lg);
  z-index: 10;
}
.empty-menu { padding: var(--space-3); font-size: var(--fs-sm); color: var(--text-dim); text-align: center; }
.attach-item {
  display: flex;
  flex-direction: column;
  gap: 2px;
  width: 100%;
  background: none;
  border: none;
  border-bottom: 1px solid var(--border);
  padding: var(--space-2) var(--space-3);
  text-align: left;
  cursor: pointer;
  color: var(--text);
  font-family: inherit;
  transition: background var(--t-fast);
}
.attach-item:last-child { border-bottom: none; }
.attach-item:hover { background: var(--surface-2); }
.attach-date { font-family: var(--font-mono); font-size: var(--fs-xs); color: var(--text-mute); }
.attach-preview { font-size: var(--fs-sm); }

.output {
  margin: 0;
  padding: var(--space-5);
  white-space: pre-wrap;
  font-family: inherit;
  font-size: var(--fs-base);
  line-height: 1.6;
  color: var(--text);
  background: var(--bg-2);
  border-radius: 0 0 var(--radius-lg) var(--radius-lg);
  overflow-x: auto;
}

.history h3 {
  margin: 0 0 var(--space-3);
  font-size: var(--fs-xs);
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--text-mute);
  font-weight: 600;
}
.empty-history { color: var(--text-dim); font-size: var(--fs-base); padding: var(--space-2) 0; }
.history-list { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 4px; }
.history-item {
  display: flex;
  gap: var(--space-2);
  align-items: center;
  padding: var(--space-2) var(--space-3);
  border-radius: var(--radius-md);
  cursor: pointer;
  font-size: var(--fs-sm);
  border: 1px solid transparent;
  transition: background var(--t-fast), border-color var(--t-fast);
}
.history-item:hover { background: var(--surface-2); border-color: var(--border); }
.history-item.active { background: var(--accent-dim); border-color: var(--border-accent); }
.h-kind { font-weight: 600; color: var(--text); }
.h-sep { color: var(--text-mute); }
.h-source { color: var(--text-dim); }
.h-source.claude { color: var(--accent-strong); }
.h-ts { font-family: var(--font-mono); color: var(--text-mute); }
.h-saved { color: var(--success); margin-left: auto; }
</style>
