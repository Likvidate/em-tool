<script setup lang="ts">
import { computed, onMounted, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useReportsStore } from "../stores/reports";
import { useWeekRatingsStore } from "../stores/week-ratings";
import ColorStrip from "../components/ColorStrip.vue";
import type { Color } from "../lib/colors";

const route = useRoute();
const router = useRouter();
const reports = useReportsStore();
const ratings = useWeekRatingsStore();

const reportId = computed(() => Number(route.params.id));

const report = computed(() => reports.byId(reportId.value));

const ratingsForReport = computed(() => {
  return Object.values(ratings.byKey)
    .filter((r) => r.reportId === reportId.value)
    .sort((a, b) => a.isoWeek.localeCompare(b.isoWeek));
});

const stripCells = computed(() =>
  ratingsForReport.value.map((r) => ({
    isoWeek: r.isoWeek,
    color: r.color as Color,
    title: `${r.isoWeek}: ${r.color}${r.notes ? " — " + r.notes : ""}`,
  })),
);

const counts = computed(() => {
  const out: Record<Color, number> = { red: 0, yellow: 0, grey: 0, green: 0, blue: 0 };
  for (const r of ratingsForReport.value) out[r.color as Color] += 1;
  return out;
});

onMounted(async () => {
  if (!reports.loaded) await reports.load(true);
  await ratings.loadForReport(reportId.value);
});

watch(reportId, async (id) => {
  await ratings.loadForReport(id);
});
</script>

<template>
  <div class="timeline" v-if="report">
    <header class="head">
      <div>
        <button class="back" @click="router.push('/reports')">← Team members</button>
        <h2>{{ report.name }}</h2>
        <p class="sub">
          {{ report.role ?? "—" }} ·
          1:1 every {{ report.oneOnOneCadenceDays }}d ·
          joined {{ report.startDate ?? "—" }}
        </p>
      </div>
    </header>

    <div class="stats">
      <div class="stat"><strong>{{ ratingsForReport.length }}</strong><span>weeks</span></div>
      <div class="stat"><span class="sw green"></span><strong>{{ counts.green }}</strong></div>
      <div class="stat"><span class="sw yellow"></span><strong>{{ counts.yellow }}</strong></div>
      <div class="stat"><span class="sw red"></span><strong>{{ counts.red }}</strong></div>
      <div class="stat"><span class="sw blue"></span><strong>{{ counts.blue }}</strong></div>
      <div class="stat"><span class="sw grey"></span><strong>{{ counts.grey }}</strong></div>
    </div>

    <ColorStrip :cells="stripCells" />

    <div class="feed">
      <div v-if="ratingsForReport.length === 0" class="empty">
        No ratings yet. Head to Weekly capture to start tracking.
      </div>
      <div v-for="r in [...ratingsForReport].reverse()" :key="r.id" class="entry">
        <div class="week">{{ r.isoWeek }}</div>
        <div class="sw" :class="r.color"></div>
        <div class="notes">{{ r.notes ?? "—" }}</div>
      </div>
    </div>
  </div>
  <div v-else class="loading">Loading…</div>
</template>

<style scoped>
.timeline { max-width: 900px; }
.head { padding: 14px; background: var(--surface); border: 1px solid var(--border); border-radius: 6px 6px 0 0; }
.back { background: none; border: none; color: var(--text-dim); font-size: 12px; cursor: pointer; margin-bottom: 4px; padding: 0; }
h2 { margin: 0; }
.sub { margin: 3px 0 0; font-size: 12px; opacity: 0.6; }
.stats {
  display: flex; gap: 18px;
  padding: 10px 14px;
  background: #141414; border-left: 1px solid var(--border); border-right: 1px solid var(--border);
  font-size: 12px; align-items: center;
}
.stat { display: flex; align-items: center; gap: 4px; }
.stat strong { color: var(--text); }
.sw { width: 14px; height: 14px; border-radius: 3px; display: inline-block; }
.sw.red    { background: #ef4444; }
.sw.yellow { background: #facc15; }
.sw.grey   { background: #6b7280; }
.sw.green  { background: #4ade80; }
.sw.blue   { background: #3b82f6; }

.feed { background: #141414; border: 1px solid var(--border); border-top: none; border-radius: 0 0 6px 6px; }
.entry {
  display: grid; grid-template-columns: 90px 20px 1fr; gap: 10px;
  padding: 10px 14px; border-bottom: 1px solid #222;
  font-size: 13px; align-items: center;
}
.entry:last-child { border-bottom: none; }
.week { font-family: monospace; opacity: 0.55; }
.empty, .loading { padding: 32px; text-align: center; color: var(--text-dim); }
</style>
