<script setup lang="ts">
import { onMounted, ref } from "vue";
import {
  settingsApi, weekRatingsApi, oneOnOnesApi,
  actionItemsApi, reviewsApi,
} from "../lib/invoke";
import { useReportsStore } from "../stores/reports";
import ConfirmModal from "../components/ConfirmModal.vue";
import { currentIsoWeek, addWeeks, formatIsoWeek } from "../lib/iso-week";

const reportsStore = useReportsStore();

const hasKey = ref(false);
const keyInput = ref("");
const saving = ref(false);
const saveError = ref<string | null>(null);
const justSaved = ref(false);
const showClearConfirm = ref(false);

const seeding = ref(false);
const seedResult = ref<string | null>(null);

async function refresh() {
  hasKey.value = await settingsApi.hasApiKey();
}

async function saveKey() {
  if (!keyInput.value.trim()) return;
  saving.value = true;
  saveError.value = null;
  try {
    await settingsApi.setApiKey(keyInput.value.trim());
    keyInput.value = "";
    await refresh();
    justSaved.value = true;
    setTimeout(() => { justSaved.value = false; }, 2000);
  } catch (e: unknown) {
    saveError.value = e instanceof Error ? e.message : String(e);
  } finally {
    saving.value = false;
  }
}

async function clearKey() {
  showClearConfirm.value = false;
  await settingsApi.setApiKey(null);
  await refresh();
}

async function seedDemoData() {
  seeding.value = true;
  seedResult.value = null;
  try {
    const demo = await reportsStore.create({
      name: "Alex Demo",
      role: "Senior Backend Engineer",
      startDate: "2024-06-01",
      oneOnOneCadenceDays: 14,
      notes: "Demo team member — seeded for testing the plan generator.",
    });

    const now = currentIsoWeek();
    const weekNotes: Array<[number, "red" | "yellow" | "grey" | "green" | "blue", string | null]> = [
      [-11, "green", "Landed the auth refactor RFC early."],
      [-10, "green", "Mentored Catalina on her first design doc."],
      [-9, "yellow", "Blocked 3 days on API contract with platform team."],
      [-8, "red", "Heated PR thread with Dmitri over schema choice."],
      [-7, "yellow", "Things calming down after the PR incident."],
      [-6, "green", "Shipped rate-limiter service. Strong delivery."],
      [-5, "blue", "Led first cross-team architecture review. Big growth moment."],
      [-4, "green", "Steady week — good code review turnaround."],
      [-3, "green", "Helped Bohdan unblock the CI flakiness."],
      [-2, "yellow", "Mentioned feeling stretched in 1:1."],
      [-1, "green", "Shipped payments API a day early."],
      [0, "green", "Wants to discuss staff-eng career path."],
    ];

    for (const [offset, color, notes] of weekNotes) {
      const w = formatIsoWeek(addWeeks(now, offset));
      await weekRatingsApi.upsert({
        reportId: demo.id, isoWeek: w, color, notes,
      });
    }

    const nowSecs = Math.floor(Date.now() / 1000);
    const oneMonthAgo = nowSecs - 30 * 24 * 60 * 60;
    const oneWeekAgo = nowSecs - 7 * 24 * 60 * 60;

    const m1 = await oneOnOnesApi.create({
      reportId: demo.id,
      occurredAt: oneMonthAgo,
      agendaMd: "Discuss Q2 goals, PR-thread incident with Dmitri, feedback from arch review.",
      notesMd: "Agreed: Alex leads the rate-limiter refactor as a visibility piece. Talked about tone in PR reviews.",
    });
    const m2 = await oneOnOnesApi.create({
      reportId: demo.id,
      occurredAt: oneWeekAgo,
      agendaMd: "Staff-eng career path discussion, handoff from rate-limiter project.",
      notesMd: "Alex wants clearer path to staff. I owe them a growth-plan doc by end of month.",
    });

    await actionItemsApi.create({
      reportId: demo.id, oneOnOneId: m2.id,
      text: "Draft growth-plan doc for Alex's staff-eng trajectory",
      owner: "me", dueDate: null,
    });
    await actionItemsApi.create({
      reportId: demo.id, oneOnOneId: m2.id,
      text: "Propose architecture for the new notifications service",
      owner: "them",
      dueDate: formatNextDate(14),
    });
    const actionDone = await actionItemsApi.create({
      reportId: demo.id, oneOnOneId: m1.id,
      text: "Land the rate-limiter RFC",
      owner: "them", dueDate: null,
    });
    await actionItemsApi.toggle(actionDone.id);

    await reviewsApi.create({
      reportId: demo.id,
      period: "Q1 2026",
      rating: "Exceeds",
      strengthsMd: "Strong technical delivery, drives RFCs end-to-end, excellent written communication.",
      devAreasMd: "Cross-team coordination (escalation), tone in code reviews, mentoring junior engineers.",
      goalsMd: "Lead one cross-team project. Mentor 1-2 junior engineers. Draft a staff-eng growth plan.",
      occurredAt: nowSecs - 45 * 24 * 60 * 60,
    });

    seedResult.value = `Created "Alex Demo" with 12 weeks of ratings, 2 past 1:1s, 3 action items (1 completed), and 1 performance review.`;
  } catch (e: unknown) {
    seedResult.value = `Error: ${e instanceof Error ? e.message : String(e)}`;
  } finally {
    seeding.value = false;
  }
}

function formatNextDate(daysAhead: number): string {
  const d = new Date(Date.now() + daysAhead * 24 * 60 * 60 * 1000);
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${d.getUTCFullYear()}-${pad(d.getUTCMonth() + 1)}-${pad(d.getUTCDate())}`;
}

onMounted(refresh);
</script>

<template>
  <div class="settings">
    <header class="page-header">
      <div>
        <h2 class="page-title">Settings</h2>
        <p class="page-subtitle">Configure your vault and integrations.</p>
      </div>
    </header>

    <section class="card card-body">
      <h3>Anthropic API key</h3>
      <p class="sub">
        Required for Claude-powered plan generation.
        Get one at
        <a href="https://console.anthropic.com" target="_blank" rel="noopener">
          console.anthropic.com</a>.
      </p>

      <div class="status-line">
        <span v-if="hasKey" class="badge badge-accent">Configured ✓</span>
        <span v-else class="badge">Not configured</span>
      </div>

      <div class="input-row">
        <input
          v-model="keyInput"
          type="password"
          class="field-input key-input"
          placeholder="sk-ant-api03-..."
          autocomplete="off"
          :disabled="saving"
          @keyup.enter="saveKey"
        />
        <button
          type="button"
          class="btn btn-primary"
          :disabled="saving || !keyInput.trim()"
          @click="saveKey"
        >
          {{ saving ? "Saving…" : "Save" }}
        </button>
        <button
          type="button"
          class="btn btn-danger"
          :disabled="!hasKey || saving"
          @click="showClearConfirm = true"
        >
          Clear
        </button>
      </div>

      <div v-if="saveError" class="error-banner">{{ saveError }}</div>
      <div v-if="justSaved" class="success-banner">API key saved.</div>
    </section>

    <section class="card card-body">
      <h3>Vault</h3>
      <p class="sub">
        Your vault is encrypted with your password. If you forget the password
        there is no recovery — by design.
      </p>
      <div class="input-row">
        <button type="button" class="btn btn-secondary" disabled>
          Change password
        </button>
        <span class="note">Password change coming in a future update.</span>
      </div>
    </section>

    <section class="card card-body">
      <h3>Developer</h3>
      <p class="sub">
        Seeds a demo team member named "Alex Demo" with a realistic history so
        you can see the plan generator produce meaningful output. Safe to run
        multiple times — it'll just add another "Alex Demo" (delete the extras
        from the Team members list when you're done).
      </p>
      <div class="input-row">
        <button
          type="button"
          class="btn btn-secondary"
          :disabled="seeding"
          @click="seedDemoData"
        >
          {{ seeding ? "Seeding…" : "Create demo team member" }}
        </button>
      </div>
      <div v-if="seedResult" :class="seedResult.startsWith('Error') ? 'error-banner' : 'success-banner'">
        {{ seedResult }}
      </div>
    </section>

    <section class="card card-body">
      <h3>About</h3>
      <p class="about-line">EM Tool v0.1.0</p>
      <p class="about-line">
        Local-only — no data leaves your machine except when generating plans with Claude.
      </p>
      <p class="about-line">
        <a href="https://github.com/Likvidate/em-tool" target="_blank" rel="noopener">
          github.com/Likvidate/em-tool
        </a>
      </p>
    </section>

    <ConfirmModal
      v-if="showClearConfirm"
      title="Clear API key?"
      message="Claude generation will stop working until you add a new key."
      confirm-label="Clear key"
      variant="danger"
      @confirm="clearKey"
      @cancel="showClearConfirm = false"
    />
  </div>
</template>

<style scoped>
.settings {
  max-width: 800px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: var(--space-5);
}

h3 {
  margin: 0 0 6px;
  font-size: var(--fs-md);
  font-weight: 600;
}
.sub {
  margin: 0 0 var(--space-4);
  font-size: var(--fs-sm);
  color: var(--text-dim);
  line-height: 1.6;
}
.sub a { color: var(--accent-strong); text-decoration: none; }
.sub a:hover { text-decoration: underline; }

.status-line { margin-bottom: var(--space-3); }

.input-row {
  display: flex;
  gap: var(--space-2);
  align-items: center;
  flex-wrap: wrap;
}
.key-input {
  flex: 1;
  min-width: 260px;
  font-family: var(--font-mono);
}

.note {
  font-size: var(--fs-xs);
  font-style: italic;
  color: var(--text-mute);
}

.error-banner {
  margin-top: var(--space-3);
  padding: 8px 12px;
  background: rgba(239, 68, 68, 0.08);
  border: 1px solid #5a2a2a;
  color: #e5a8a8;
  border-radius: var(--radius-md);
  font-size: var(--fs-sm);
}
.success-banner {
  margin-top: var(--space-3);
  padding: 8px 12px;
  background: rgba(74, 222, 128, 0.08);
  border: 1px solid #2f5a3a;
  color: var(--success);
  border-radius: var(--radius-md);
  font-size: var(--fs-sm);
}

.about-line {
  margin: 0 0 6px;
  font-size: var(--fs-base);
  color: var(--text);
}
.about-line:last-child { margin-bottom: 0; }
.about-line a { color: var(--accent-strong); text-decoration: none; }
.about-line a:hover { text-decoration: underline; }
</style>
