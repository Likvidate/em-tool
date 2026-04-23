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
    <header class="page-head">
      <h2>Weekly capture</h2>
      <WeekNav :label="label" :is-current="isCurrent" @prev="prev" @next="next" @jump-to-current="toCurrent" />
    </header>

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
        class="note"
        @blur="setNotes(null, ($event.target as HTMLInputElement).value)"
      />
    </div>

    <div v-if="reports.active.length === 0" class="empty">
      <p>Add reports first to capture weekly ratings.</p>
      <router-link to="/reports" class="link">Go to Reports →</router-link>
    </div>

    <div v-else class="grid">
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
          class="note"
          @blur="setNotes(r.id, ($event.target as HTMLInputElement).value)"
        />
        <div class="cadence">every {{ r.oneOnOneCadenceDays }}d</div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.capture { max-width: 960px; }
.page-head { display: flex; justify-content: space-between; align-items: center; margin-bottom: 18px; gap: 18px; }
h2 { margin: 0; font-size: 20px; }
.team-row {
  display: grid;
  grid-template-columns: 180px auto 1fr;
  gap: 12px; align-items: center;
  padding: 12px 14px;
  background: #1f2937;
  border-left: 3px solid var(--blue);
  border-radius: 6px;
  margin-bottom: 16px;
}
.team-label { font-size: 11px; text-transform: uppercase; letter-spacing: 0.08em; opacity: 0.75; font-weight: 600; }

.grid { background: var(--surface); border: 1px solid var(--border); border-radius: 6px; }
.row {
  display: grid;
  grid-template-columns: 180px 180px 1fr 80px;
  gap: 12px; align-items: center;
  padding: 10px 12px;
  border-bottom: 1px solid var(--border);
}
.row:last-child { border-bottom: none; }
.person .name { font-weight: 600; }
.person .role { font-size: 12px; opacity: 0.6; }
.note {
  background: var(--bg); border: 1px solid var(--border); color: var(--text);
  padding: 5px 8px; border-radius: 4px; font-size: 12px; font-family: inherit;
}
.cadence { font-size: 11px; opacity: 0.5; text-align: right; }
.empty { padding: 48px 0; text-align: center; color: var(--text-dim); }
.link { color: var(--accent); text-decoration: underline; }
</style>
