<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useReportsStore } from "../stores/reports";
import { useWeekRatingsStore } from "../stores/week-ratings";
import { useOneOnOnesStore } from "../stores/one-on-ones";
import { useActionItemsStore } from "../stores/action-items";
import { useReviewsStore } from "../stores/reviews";
import ColorStrip from "../components/ColorStrip.vue";
import ActionItemList from "../components/ActionItemList.vue";
import LogOneOnOneModal from "../components/LogOneOnOneModal.vue";
import LogReviewModal from "../components/LogReviewModal.vue";
import type { Color } from "../lib/colors";
import type { WeekRating } from "../types/week-rating";
import type { OneOnOne } from "../types/one-on-one";
import type { PerformanceReview } from "../types/performance-review";

type FeedEntry =
  | { kind: "week"; ts: number; data: WeekRating }
  | { kind: "one_on_one"; ts: number; data: OneOnOne }
  | { kind: "review"; ts: number; data: PerformanceReview };

const route = useRoute();
const router = useRouter();
const reports = useReportsStore();
const ratings = useWeekRatingsStore();
const oneOnOnes = useOneOnOnesStore();
const actionItems = useActionItemsStore();
const reviews = useReviewsStore();

const reportId = computed(() => Number(route.params.id));

const report = computed(() => reports.byId(reportId.value));

const showLog1on1 = ref(false);
const showLogReview = ref(false);
const editing1on1 = ref<OneOnOne | null>(null);
const editingReview = ref<PerformanceReview | null>(null);

function onEntryClick(e: FeedEntry) {
  if (e.kind === "one_on_one") editing1on1.value = e.data;
  else if (e.kind === "review") editingReview.value = e.data;
}

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

const openActions = computed(() =>
  actionItems.forReport(reportId.value).filter((a) => !a.completedAt),
);

const feedEntries = computed<FeedEntry[]>(() => {
  const entries: FeedEntry[] = [];
  for (const wr of ratingsForReport.value) {
    entries.push({ kind: "week", ts: wr.createdAt, data: wr });
  }
  for (const m of oneOnOnes.forReport(reportId.value)) {
    entries.push({ kind: "one_on_one", ts: m.occurredAt, data: m });
  }
  for (const r of reviews.forReport(reportId.value)) {
    entries.push({ kind: "review", ts: r.occurredAt, data: r });
  }
  entries.sort((a, b) => b.ts - a.ts);
  return entries;
});

function formatTs(unixSecs: number): string {
  const d = new Date(unixSecs * 1000);
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${d.getUTCFullYear()}-${pad(d.getUTCMonth() + 1)}-${pad(d.getUTCDate())}`;
}

function preview(md: string | null, max = 120): string {
  if (!md) return "";
  const trimmed = md.trim();
  if (trimmed.length <= max) return trimmed;
  return trimmed.slice(0, max).trimEnd() + "…";
}

async function loadAll(id: number) {
  await Promise.all([
    ratings.loadForReport(id),
    oneOnOnes.loadForReport(id),
    actionItems.loadForReport(id),
    reviews.loadForReport(id),
  ]);
}

async function onCreatedOneOnOne() {
  await Promise.all([
    oneOnOnes.loadForReport(reportId.value),
    actionItems.loadForReport(reportId.value),
  ]);
}

async function onCreatedReview() {
  await reviews.loadForReport(reportId.value);
}

onMounted(async () => {
  if (!reports.loaded) await reports.load(true);
  await loadAll(reportId.value);
});

watch(reportId, async (id) => {
  await loadAll(id);
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
      <div class="person-actions">
        <button @click="showLog1on1 = true">+ Log 1:1</button>
        <button @click="showLogReview = true">+ Log review</button>
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

    <section class="open-actions" v-if="openActions.length > 0">
      <h3>Open action items</h3>
      <ActionItemList
        :items="openActions"
        @toggle="(id) => actionItems.toggle(id, reportId)"
        @delete="(id) => actionItems.remove(id, reportId)"
      />
    </section>

    <div class="feed">
      <div v-if="feedEntries.length === 0" class="empty">
        Nothing logged yet. Capture a weekly rating, log a 1:1, or record a review.
      </div>
      <div
        v-for="e in feedEntries"
        :key="e.kind + ':' + e.data.id"
        class="entry"
        :class="[e.kind, { clickable: e.kind === 'one_on_one' || e.kind === 'review' }]"
        @click="onEntryClick(e)"
      >
        <div class="date">{{ formatTs(e.ts) }}</div>
        <div class="tag">
          {{ e.kind === "one_on_one" ? "1:1" : e.kind === "review" ? "Review" : "Week" }}
        </div>
        <div class="body">
          <template v-if="e.kind === 'week'">
            <div class="week-line">
              <span class="sw" :class="e.data.color"></span>
              <span class="iso">{{ e.data.isoWeek }}</span>
              <span class="notes">{{ e.data.notes ?? "—" }}</span>
            </div>
          </template>
          <template v-else-if="e.kind === 'one_on_one'">
            <div class="agenda">
              {{ e.data.agendaMd ? preview(e.data.agendaMd) : "(no agenda)" }}
            </div>
          </template>
          <template v-else>
            <div class="review-head">
              {{ e.data.period }}<template v-if="e.data.rating"> · {{ e.data.rating }}</template>
            </div>
            <div v-if="e.data.devAreasMd" class="dev-areas">
              {{ preview(e.data.devAreasMd) }}
            </div>
          </template>
        </div>
      </div>
    </div>

    <LogOneOnOneModal
      v-if="showLog1on1"
      :report-id="reportId"
      @close="showLog1on1 = false"
      @created="onCreatedOneOnOne"
    />
    <LogReviewModal
      v-if="showLogReview"
      :report-id="reportId"
      @close="showLogReview = false"
      @created="onCreatedReview"
    />
    <LogOneOnOneModal
      v-if="editing1on1"
      :report-id="reportId"
      :existing="editing1on1"
      @close="editing1on1 = null"
      @created="onCreatedOneOnOne"
    />
    <LogReviewModal
      v-if="editingReview"
      :report-id="reportId"
      :existing="editingReview"
      @close="editingReview = null"
      @created="onCreatedReview"
    />
  </div>
  <div v-else class="loading">Loading…</div>
</template>

<style scoped>
.timeline { max-width: 900px; }
.head {
  display: flex; justify-content: space-between; align-items: flex-start; gap: 12px;
  padding: 14px; background: var(--surface); border: 1px solid var(--border);
  border-radius: 6px 6px 0 0;
}
.back { background: none; border: none; color: var(--text-dim); font-size: 12px; cursor: pointer; margin-bottom: 4px; padding: 0; }
h2 { margin: 0; }
.sub { margin: 3px 0 0; font-size: 12px; opacity: 0.6; }
.person-actions { display: flex; gap: 8px; flex-shrink: 0; }
.person-actions button {
  background: var(--surface-2, #1f2937); color: var(--text);
  border: 1px solid var(--border); border-radius: 4px;
  padding: 6px 10px; font-size: 12px; font-family: inherit; cursor: pointer;
}
.person-actions button:hover { border-color: var(--accent); color: var(--accent); }

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

.open-actions {
  background: #141414;
  border-left: 1px solid var(--border);
  border-right: 1px solid var(--border);
  border-top: 1px solid #222;
  padding: 12px 14px;
}
.open-actions h3 {
  margin: 0 0 8px; font-size: 12px; text-transform: uppercase;
  letter-spacing: 0.5px; color: var(--text-dim); font-weight: 600;
}

.feed { background: #141414; border: 1px solid var(--border); border-top: none; border-radius: 0 0 6px 6px; }
.entry {
  display: grid; grid-template-columns: 90px 72px 1fr; gap: 10px;
  padding: 10px 14px; border-bottom: 1px solid #222;
  font-size: 13px; align-items: center;
}
.entry:last-child { border-bottom: none; }
.date { font-family: monospace; opacity: 0.55; font-size: 12px; }
.tag {
  display: inline-block;
  padding: 2px 8px;
  border-radius: 3px;
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  font-weight: 600;
  text-align: center;
  justify-self: start;
}
.entry.week .tag { background: #374151; color: var(--text-dim); }
.entry.one_on_one .tag { background: #1e40af; color: #fff; }
.entry.review .tag { background: #7c2d12; color: #fff; }
.entry.clickable { cursor: pointer; transition: background 120ms; }
.entry.clickable:hover { background: var(--surface-2, #1f2937); }

.week-line { display: flex; align-items: center; gap: 10px; }
.week-line .iso { font-family: monospace; opacity: 0.7; font-size: 12px; }
.week-line .notes { color: var(--text); }
.agenda { color: var(--text); line-height: 1.4; }
.review-head { color: var(--text); font-weight: 500; }
.dev-areas { color: var(--text-dim); font-size: 12px; margin-top: 2px; line-height: 1.4; }

.empty, .loading { padding: 32px; text-align: center; color: var(--text-dim); }
</style>
