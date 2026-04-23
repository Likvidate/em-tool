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
    <header class="page-head">
      <h2>Team heatmap</h2>
      <div class="range">
        <label>Range:</label>
        <select :value="rangeWeeks" @change="(e) => changeRange(Number((e.target as HTMLSelectElement).value))">
          <option :value="13">Last 13 weeks</option>
          <option :value="26">Last 26 weeks</option>
          <option :value="52">Last 52 weeks</option>
        </select>
      </div>
    </header>

    <div class="grid-wrap">
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
.heatmap { max-width: 100%; }
.page-head { display: flex; justify-content: space-between; align-items: center; margin-bottom: 14px; }
h2 { margin: 0; font-size: 20px; }
.range { display: flex; align-items: center; gap: 8px; font-size: 12px; color: var(--text-dim); }
.range select { background: var(--bg); border: 1px solid var(--border); color: var(--text); padding: 4px 8px; border-radius: 4px; font-size: 12px; }

.grid-wrap { background: #141414; border: 1px solid var(--border); border-radius: 6px; padding: 14px; overflow-x: auto; }
.grid { min-width: 600px; display: flex; flex-direction: column; gap: 2px; }
.row {
  display: grid; grid-template-columns: 140px 1fr;
  gap: 10px; align-items: center;
}
.cells { display: flex; gap: 2px; flex: 1; }
.name { font-size: 12px; opacity: 0.8; padding-right: 8px; }
.row.team .name { font-weight: 700; color: #93c5fd; }
.sep { height: 1px; background: #333; margin: 6px 0; }
.cell { flex: 1; min-width: 6px; height: 22px; border-radius: 2px; }
.cell.red    { background: #ef4444; }
.cell.yellow { background: #facc15; }
.cell.grey   { background: #6b7280; }
.cell.green  { background: #4ade80; }
.cell.blue   { background: #3b82f6; }
.cell.none   { background: #1a1a1a; }
</style>
