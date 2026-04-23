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
    <header class="head">
      <h2>Plan generator</h2>
      <p class="sub">Draft a 1:1 prep or review prep from recent signals.</p>
    </header>

    <section class="card controls">
      <div class="row">
        <label class="field">
          <span class="field-label">Team member</span>
          <select
            v-model="selectedReportId"
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
          <label>
            <span class="mini-label">From</span>
            <input v-model="customFrom" placeholder="2025-W01" />
          </label>
          <label>
            <span class="mini-label">To</span>
            <input v-model="customTo" placeholder="2025-W12" />
          </label>
        </div>
      </div>

      <div class="generate-row">
        <div class="claude-wrap">
          <button
            type="button"
            class="primary"
            :disabled="!hasApiKey || plans.generating || !selectedReportId"
            @click="doGenerate('claude')"
          >
            <span v-if="plans.generating" class="spinner"></span>
            {{ plans.generating ? "Generating…" : "✨ Generate with Claude" }}
          </button>
          <span v-if="!hasApiKey" class="hint">
            Add an Anthropic API key in Settings
          </span>
        </div>
        <button
          type="button"
          class="secondary"
          :disabled="plans.generating || !selectedReportId"
          @click="doGenerate('template')"
        >
          <span v-if="plans.generating" class="spinner"></span>
          {{ plans.generating ? "Generating…" : "Generate from template" }}
        </button>
      </div>

      <div v-if="error" class="error-banner">{{ error }}</div>
    </section>

    <section class="card output-card">
      <div v-if="!currentPlan" class="empty-output">
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
            <button type="button" class="secondary" @click="copyOutput">
              {{ copied ? "Copied!" : "Copy markdown" }}
            </button>
            <div class="attach-wrap">
              <button
                type="button"
                class="secondary"
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

    <section class="card history">
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
.plan-gen { max-width: 900px; display: flex; flex-direction: column; gap: 16px; }

.head h2 { margin: 0 0 4px; }
.sub { margin: 0; color: var(--text-dim); font-size: 13px; }

.card {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 16px;
}

.controls { display: flex; flex-direction: column; gap: 16px; }
.row { display: flex; gap: 16px; flex-wrap: wrap; }

.field { display: flex; flex-direction: column; gap: 6px; flex: 1; min-width: 220px; }
.field-label {
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--text-dim);
  font-weight: 600;
}

select, input {
  background: #141414;
  color: var(--text);
  border: 1px solid var(--border);
  border-radius: 4px;
  padding: 7px 10px;
  font-size: 13px;
  font-family: inherit;
}
select:disabled { opacity: 0.5; cursor: not-allowed; }
select:focus, input:focus { outline: none; border-color: var(--accent); }

.toggle-group { display: inline-flex; gap: 0; }
.toggle {
  background: #141414;
  color: var(--text);
  border: 1px solid var(--border);
  padding: 7px 14px;
  font-size: 13px;
  cursor: pointer;
  font-family: inherit;
}
.toggle:first-child { border-radius: 4px 0 0 4px; }
.toggle:last-child { border-radius: 0 4px 4px 0; border-left: none; }
.toggle.active { background: var(--accent); color: #fff; border-color: var(--accent); }
.toggle:hover:not(.active) { border-color: var(--accent); color: var(--accent); }

.pills { display: inline-flex; gap: 6px; flex-wrap: wrap; }
.pill {
  background: #141414;
  color: var(--text-dim);
  border: 1px solid var(--border);
  border-radius: 999px;
  padding: 5px 12px;
  font-size: 12px;
  cursor: pointer;
  font-family: inherit;
}
.pill:hover:not(.active) { border-color: var(--accent); color: var(--accent); }
.pill.active { background: var(--accent); color: #fff; border-color: var(--accent); }

.custom-range { display: flex; gap: 10px; margin-top: 8px; }
.custom-range label { display: flex; flex-direction: column; gap: 4px; }
.mini-label { font-size: 10px; color: var(--text-dim); text-transform: uppercase; letter-spacing: 0.5px; }
.custom-range input { width: 130px; font-family: monospace; }

.generate-row { display: flex; gap: 10px; align-items: flex-start; flex-wrap: wrap; }
.claude-wrap { display: flex; flex-direction: column; gap: 4px; }
.hint { font-size: 11px; color: var(--text-dim); font-style: italic; padding: 0 2px; }

button.primary, button.secondary {
  padding: 8px 14px;
  border-radius: 4px;
  font-size: 13px;
  cursor: pointer;
  font-family: inherit;
  display: inline-flex;
  align-items: center;
  gap: 8px;
}
button.primary {
  background: var(--accent);
  color: #fff;
  border: 1px solid var(--accent);
}
button.primary:disabled {
  background: #374151;
  border-color: var(--border);
  color: var(--text-dim);
  cursor: not-allowed;
}
button.secondary {
  background: #141414;
  color: var(--text);
  border: 1px solid var(--border);
}
button.secondary:hover:not(:disabled) { border-color: var(--accent); color: var(--accent); }
button.secondary:disabled { opacity: 0.5; cursor: not-allowed; }

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
  border-radius: 4px;
  font-size: 12px;
}

.output-card { padding: 0; }
.empty-output {
  padding: 48px 16px;
  text-align: center;
  color: var(--text-dim);
  font-size: 13px;
}
.output-head {
  display: flex; justify-content: space-between; align-items: flex-start;
  gap: 10px; flex-wrap: wrap;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border);
}
.output-meta { display: flex; gap: 10px; align-items: center; flex-wrap: wrap; }
.badge {
  font-size: 10px;
  padding: 3px 8px;
  border-radius: 3px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  font-weight: 600;
}
.badge.claude { background: #4c1d95; color: #e9d5ff; }
.badge.template { background: #374151; color: var(--text-dim); }
.kind-tag {
  font-size: 12px;
  color: var(--text-dim);
}
.ts {
  font-family: monospace;
  font-size: 11px;
  color: var(--text-dim);
}
.saved {
  font-size: 11px;
  color: #4ade80;
}
.output-actions { display: flex; gap: 8px; position: relative; }
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
  border-radius: 4px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5);
  z-index: 10;
}
.empty-menu { padding: 12px; font-size: 12px; color: var(--text-dim); text-align: center; }
.attach-item {
  display: flex;
  flex-direction: column;
  gap: 2px;
  width: 100%;
  background: none;
  border: none;
  border-bottom: 1px solid #222;
  padding: 8px 12px;
  text-align: left;
  cursor: pointer;
  color: var(--text);
  font-family: inherit;
}
.attach-item:last-child { border-bottom: none; }
.attach-item:hover { background: #1f2937; }
.attach-date { font-family: monospace; font-size: 11px; color: var(--text-dim); }
.attach-preview { font-size: 12px; }

.output {
  margin: 0;
  padding: 16px;
  white-space: pre-wrap;
  font-family: inherit;
  font-size: 13px;
  line-height: 1.55;
  color: var(--text);
  background: #141414;
  border-radius: 0 0 6px 6px;
  overflow-x: auto;
}

.history h3 {
  margin: 0 0 10px;
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--text-dim);
  font-weight: 600;
}
.empty-history { color: var(--text-dim); font-size: 13px; padding: 8px 0; }
.history-list { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 4px; }
.history-item {
  display: flex;
  gap: 8px;
  align-items: center;
  padding: 8px 10px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  border: 1px solid transparent;
}
.history-item:hover { background: #141414; border-color: var(--border); }
.history-item.active { background: #141414; border-color: var(--accent); }
.h-kind { font-weight: 600; color: var(--text); }
.h-sep { color: var(--text-dim); }
.h-source { color: var(--text-dim); }
.h-source.claude { color: #c4b5fd; }
.h-ts { font-family: monospace; color: var(--text-dim); }
.h-saved { color: #4ade80; margin-left: auto; }
</style>
