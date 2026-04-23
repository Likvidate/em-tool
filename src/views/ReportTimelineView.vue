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

function initials(name: string): string {
  const parts = name.trim().split(/\s+/);
  if (parts.length === 0) return "?";
  if (parts.length === 1) return parts[0].slice(0, 2).toUpperCase();
  return (parts[0][0] + parts[parts.length - 1][0]).toUpperCase();
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
    <button class="back btn btn-ghost btn-sm" @click="router.push('/reports')">← Team members</button>

    <div class="card hero">
      <div class="hero-head">
        <div class="person-head">
          <div class="avatar">{{ initials(report.name) }}</div>
          <div>
            <h2 class="page-title">{{ report.name }}</h2>
            <p class="page-subtitle">
              {{ report.role ?? "—" }}
              <span class="dot">·</span>
              1:1 every {{ report.oneOnOneCadenceDays }}d
              <span class="dot">·</span>
              joined {{ report.startDate ?? "—" }}
            </p>
          </div>
        </div>
        <div class="person-actions">
          <button class="btn btn-primary" @click="showLog1on1 = true">+ Log 1:1</button>
          <button class="btn btn-secondary" @click="showLogReview = true">+ Log review</button>
        </div>
      </div>

      <div class="stats">
        <div class="stat"><strong>{{ ratingsForReport.length }}</strong><span>weeks</span></div>
        <div class="stat"><span class="sw green"></span><strong>{{ counts.green }}</strong></div>
        <div class="stat"><span class="sw yellow"></span><strong>{{ counts.yellow }}</strong></div>
        <div class="stat"><span class="sw red"></span><strong>{{ counts.red }}</strong></div>
        <div class="stat"><span class="sw blue"></span><strong>{{ counts.blue }}</strong></div>
        <div class="stat"><span class="sw grey"></span><strong>{{ counts.grey }}</strong></div>
      </div>

      <div class="strip-wrap">
        <ColorStrip :cells="stripCells" />
      </div>
    </div>

    <section v-if="openActions.length > 0" class="card open-actions">
      <h3>Open action items</h3>
      <ActionItemList
        :items="openActions"
        @toggle="(id) => actionItems.toggle(id, reportId)"
        @delete="(id) => actionItems.remove(id, reportId)"
      />
    </section>

    <div class="card feed">
      <div v-if="feedEntries.length === 0" class="empty-state">
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
.timeline {
  max-width: 1000px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: var(--space-5);
}

.back {
  align-self: flex-start;
  padding: 4px 8px;
  font-size: var(--fs-sm);
  color: var(--text-dim);
}

.hero { padding: 0; }
.hero-head {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: var(--space-4);
  padding: var(--space-5);
}
.person-head {
  display: flex;
  align-items: center;
  gap: var(--space-4);
}
.avatar {
  width: 44px;
  height: 44px;
  border-radius: 50%;
  background: linear-gradient(135deg, var(--accent), var(--accent-strong));
  color: #fff;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 600;
  font-size: var(--fs-md);
  letter-spacing: 0.02em;
  flex-shrink: 0;
  box-shadow: 0 2px 8px rgba(139, 92, 246, 0.3);
}
.dot { color: var(--text-mute); margin: 0 4px; }
.person-actions { display: flex; gap: var(--space-2); flex-shrink: 0; }

.stats {
  display: flex;
  gap: var(--space-5);
  padding: var(--space-3) var(--space-5);
  background: var(--bg-2);
  border-top: 1px solid var(--border);
  border-bottom: 1px solid var(--border);
  font-size: var(--fs-sm);
  align-items: center;
}
.stat { display: flex; align-items: center; gap: 6px; }
.stat strong { color: var(--text); font-weight: 600; }
.stat span { color: var(--text-dim); }

.sw {
  width: 14px;
  height: 14px;
  border-radius: 3px;
  display: inline-block;
}
.sw.red    { background: var(--red); }
.sw.yellow { background: var(--yellow); }
.sw.grey   { background: var(--grey); }
.sw.green  { background: var(--green); }
.sw.blue   { background: var(--blue); }

.strip-wrap { padding: var(--space-4) var(--space-5); }

.open-actions { padding: var(--space-4) var(--space-5); }
.open-actions h3 {
  margin: 0 0 var(--space-3);
  font-size: var(--fs-xs);
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--text-mute);
  font-weight: 600;
}

.feed { padding: 0; }
.entry {
  display: grid;
  grid-template-columns: 100px 80px 1fr;
  gap: var(--space-3);
  padding: var(--space-3) var(--space-5);
  border-bottom: 1px solid var(--border);
  font-size: var(--fs-base);
  align-items: center;
  transition: background var(--t-fast);
}
.entry:last-child { border-bottom: none; }
.date {
  font-family: var(--font-mono);
  color: var(--text-mute);
  font-size: var(--fs-sm);
}
.tag {
  display: inline-block;
  padding: 3px 8px;
  border-radius: 999px;
  font-size: var(--fs-xs);
  text-transform: uppercase;
  letter-spacing: 0.08em;
  font-weight: 600;
  text-align: center;
  justify-self: start;
  border: 1px solid transparent;
}
.entry.week .tag {
  background: var(--surface-2);
  color: var(--text-dim);
  border-color: var(--border);
}
.entry.one_on_one .tag {
  background: var(--accent-dim);
  color: var(--accent-strong);
  border-color: var(--border-accent);
}
.entry.review .tag {
  background: rgba(251, 146, 60, 0.12);
  color: #fdba74;
  border-color: rgba(251, 146, 60, 0.3);
}
.entry.clickable { cursor: pointer; }
.entry.clickable:hover { background: var(--surface-2); }

.week-line { display: flex; align-items: center; gap: var(--space-3); }
.week-line .iso {
  font-family: var(--font-mono);
  color: var(--text-dim);
  font-size: var(--fs-sm);
}
.week-line .notes { color: var(--text); }
.agenda { color: var(--text); line-height: 1.5; }
.review-head { color: var(--text); font-weight: 500; }
.dev-areas {
  color: var(--text-dim);
  font-size: var(--fs-sm);
  margin-top: 3px;
  line-height: 1.5;
}

.loading {
  padding: var(--space-8);
  text-align: center;
  color: var(--text-dim);
}
</style>
