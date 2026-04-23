<script setup lang="ts">
import { onMounted, watch, computed } from "vue";
import { useReportsStore } from "../stores/reports";
import { useWeekRatingsStore } from "../stores/week-ratings";
import { useCurrentWeek } from "../composables/useCurrentWeek";
import { currentIsoWeek, formatIsoWeek } from "../lib/iso-week";
import ColorSwatches from "../components/ColorSwatches.vue";
import WeekNav from "../components/WeekNav.vue";
import type { Color } from "../lib/colors";

const reports = useReportsStore();
const ratings = useWeekRatingsStore();
const { isoWeek, label, prev, next, toCurrent } = useCurrentWeek();

const isCurrent = computed(() => isoWeek.value === formatIsoWeek(currentIsoWeek()));

onMounted(async () => {
  if (!reports.loaded) await reports.load(false);
  await ratings.loadWeek(isoWeek.value);
});

watch(isoWeek, async (w) => {
  await ratings.loadWeek(w);
});

function colorFor(reportId: number | null): Color | null {
  const r = ratings.get(reportId, isoWeek.value);
  return r ? (r.color as Color) : null;
}

function notesFor(reportId: number | null): string {
  return ratings.get(reportId, isoWeek.value)?.notes ?? "";
}

async function setColor(reportId: number | null, color: Color | null) {
  if (color === null) {
    await ratings.remove(reportId, isoWeek.value);
    return;
  }
  await ratings.upsert({
    reportId,
    isoWeek: isoWeek.value,
    color,
    notes: ratings.get(reportId, isoWeek.value)?.notes ?? null,
  });
}

async function setNotes(reportId: number | null, notes: string) {
  const cur = ratings.get(reportId, isoWeek.value);
  if (!cur) return; // no rating yet — notes without color unused for MVP
  const newNotes = notes.trim() || null;
  if (cur.notes === newNotes) return;
  await ratings.upsert({
    reportId,
    isoWeek: isoWeek.value,
    color: cur.color as Color,
    notes: newNotes,
  });
}
</script>

<template>
  <div class="capture">
    <header class="page-header">
      <div>
        <h2 class="page-title">Weekly capture</h2>
        <p class="page-subtitle">Rate each person's week and jot a single-line note.</p>
      </div>
      <WeekNav :label="label" :is-current="isCurrent" @prev="prev" @next="next" @jump-to-current="toCurrent" />
    </header>

    <div class="card team-card">
      <div class="team-row">
        <div class="team-label">Team overall</div>
        <ColorSwatches
          :model-value="colorFor(null)"
          @update:model-value="(c) => setColor(null, c)"
        />
        <input
          type="text"
          :value="notesFor(null)"
          placeholder="Team note (optional)"
          class="field-input note"
          @blur="setNotes(null, ($event.target as HTMLInputElement).value)"
        />
      </div>
    </div>

    <div v-if="reports.active.length === 0" class="empty-state">
      <p>Add reports first to capture weekly ratings.</p>
      <router-link to="/reports" class="link">Go to Reports →</router-link>
    </div>

    <div v-else class="card grid">
      <div v-for="r in reports.active" :key="r.id" class="row">
        <div class="person">
          <div class="name">{{ r.name }}</div>
          <div class="role">{{ r.role ?? "" }}</div>
        </div>
        <ColorSwatches
          :model-value="colorFor(r.id)"
          @update:model-value="(c) => setColor(r.id, c)"
        />
        <input
          type="text"
          :value="notesFor(r.id)"
          placeholder="Note (optional)"
          class="field-input note"
          @blur="setNotes(r.id, ($event.target as HTMLInputElement).value)"
        />
        <div class="cadence">every {{ r.oneOnOneCadenceDays }}d</div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.capture {
  max-width: 1000px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: var(--space-5);
}

.team-card {
  border-left: 3px solid var(--blue);
  background: linear-gradient(180deg, rgba(59, 130, 246, 0.05), transparent 40%), var(--surface);
}
.team-row {
  display: grid;
  grid-template-columns: 180px auto 1fr;
  gap: var(--space-3);
  align-items: center;
  padding: var(--space-4) var(--space-5);
}
.team-label {
  font-size: var(--fs-xs);
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--text-dim);
  font-weight: 600;
}

.row {
  display: grid;
  grid-template-columns: 200px 180px 1fr 90px;
  gap: var(--space-3);
  align-items: center;
  padding: var(--space-3) var(--space-5);
  border-bottom: 1px solid var(--border);
  transition: background var(--t-fast);
}
.row:hover { background: var(--surface-2); }
.row:last-child { border-bottom: none; }
.person .name { font-weight: 600; color: var(--text); }
.person .role { font-size: var(--fs-sm); color: var(--text-dim); margin-top: 2px; }

.note {
  padding: 7px 10px;
  font-size: var(--fs-sm);
}

.cadence {
  font-size: var(--fs-xs);
  color: var(--text-mute);
  text-align: right;
  font-family: var(--font-mono);
}

.link { color: var(--accent-strong); text-decoration: none; }
.link:hover { text-decoration: underline; }
</style>
