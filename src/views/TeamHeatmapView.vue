<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useReportsStore } from "../stores/reports";
import { useWeekRatingsStore } from "../stores/week-ratings";
import { currentIsoWeek, addWeeks, weeksInRange, formatIsoWeek, type IsoWeek } from "../lib/iso-week";
import type { Color } from "../lib/colors";

const reports = useReportsStore();
const ratings = useWeekRatingsStore();

const rangeWeeks = ref(26);

const range = computed<{ weeks: IsoWeek[]; from: string; to: string }>(() => {
  const end = currentIsoWeek();
  const start = addWeeks(end, -(rangeWeeks.value - 1));
  const weeks = weeksInRange(start, end);
  return { weeks, from: formatIsoWeek(start), to: formatIsoWeek(end) };
});

onMounted(async () => {
  if (!reports.loaded) await reports.load(false);
  await ratings.loadRange(range.value.from, range.value.to);
});

async function changeRange(n: number) {
  rangeWeeks.value = n;
  await ratings.loadRange(range.value.from, range.value.to);
}

function cellColor(reportId: number | null, isoWeek: string): Color | "none" {
  const r = ratings.get(reportId, isoWeek);
  return r ? (r.color as Color) : "none";
}

function cellTitle(reportId: number | null, isoWeek: string): string {
  const r = ratings.get(reportId, isoWeek);
  if (!r) return `${isoWeek} — no rating`;
  return `${isoWeek}: ${r.color}${r.notes ? " — " + r.notes : ""}`;
}
</script>

<template>
  <div class="heatmap">
    <header class="page-header">
      <div>
        <h2 class="page-title">Team heatmap</h2>
        <p class="page-subtitle">At-a-glance view of team sentiment over time.</p>
      </div>
      <div class="range">
        <label class="range-label">Range</label>
        <select
          class="field-input range-select"
          :value="rangeWeeks"
          @change="(e) => changeRange(Number((e.target as HTMLSelectElement).value))"
        >
          <option :value="13">Last 13 weeks</option>
          <option :value="26">Last 26 weeks</option>
          <option :value="52">Last 52 weeks</option>
        </select>
      </div>
    </header>

    <div class="card grid-wrap">
      <div class="grid">
        <div class="row team">
          <div class="name">Team overall</div>
          <div class="cells">
            <div
              v-for="w in range.weeks"
              :key="w.year + '-' + w.week"
              class="cell"
              :class="cellColor(null, formatIsoWeek(w))"
              :title="cellTitle(null, formatIsoWeek(w))"
            />
          </div>
        </div>

        <div class="sep"></div>

        <div v-for="r in reports.active" :key="r.id" class="row">
          <div class="name">{{ r.name }}</div>
          <div class="cells">
            <div
              v-for="w in range.weeks"
              :key="w.year + '-' + w.week"
              class="cell"
              :class="cellColor(r.id, formatIsoWeek(w))"
              :title="cellTitle(r.id, formatIsoWeek(w))"
            />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.heatmap {
  max-width: 1200px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: var(--space-5);
}
.range { display: flex; align-items: center; gap: var(--space-2); }
.range-label {
  font-size: var(--fs-xs);
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--text-mute);
  font-weight: 600;
}
.range-select {
  width: auto;
  padding: 6px 10px;
  font-size: var(--fs-sm);
  cursor: pointer;
}

.grid-wrap {
  padding: var(--space-4);
  overflow-x: auto;
}
.grid {
  min-width: 600px;
  display: flex;
  flex-direction: column;
  gap: 3px;
}
.row {
  display: grid;
  grid-template-columns: 150px 1fr;
  gap: var(--space-3);
  align-items: center;
}
.cells { display: flex; gap: 2px; flex: 1; }
.name {
  font-size: var(--fs-sm);
  color: var(--text-dim);
  padding-right: var(--space-2);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.row.team .name { font-weight: 700; color: var(--accent-strong); }
.sep { height: 1px; background: var(--border); margin: var(--space-2) 0; }
.cell {
  flex: 1;
  min-width: 6px;
  height: 22px;
  border-radius: 2px;
  transition: transform var(--t-fast), box-shadow var(--t-fast);
  cursor: pointer;
}
.cell:hover {
  transform: scale(1.2);
  box-shadow: 0 0 0 1px var(--bg), 0 0 0 2px rgba(255, 255, 255, 0.25);
  position: relative;
  z-index: 1;
}
.cell.red    { background: var(--red); }
.cell.yellow { background: var(--yellow); }
.cell.grey   { background: var(--grey); }
.cell.green  { background: var(--green); }
.cell.blue   { background: var(--blue); }
.cell.none   { background: var(--bg-2); }
</style>
