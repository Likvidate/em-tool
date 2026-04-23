<script setup lang="ts">
import { onMounted, ref, computed } from "vue";
import { useRouter } from "vue-router";
import { useReportsStore } from "../stores/reports";
import AddReportModal from "../components/AddReportModal.vue";

const reports = useReportsStore();
const router = useRouter();
const showAdd = ref(false);
const showArchived = ref(false);

const visible = computed(() =>
  showArchived.value ? reports.items : reports.active,
);

onMounted(() => {
  if (!reports.loaded) reports.load(true);
});

function openTimeline(id: number) {
  router.push({ name: "report-timeline", params: { id: String(id) } });
}
</script>

<template>
  <div class="reports-view">
    <header class="page-head">
      <h2>Reports</h2>
      <div class="actions">
        <label class="archived-toggle">
          <input v-model="showArchived" type="checkbox" />
          <span>Show archived</span>
        </label>
        <button class="primary" @click="showAdd = true">+ Add report</button>
      </div>
    </header>

    <div v-if="reports.loading && !reports.loaded" class="empty">Loading…</div>

    <div v-else-if="visible.length === 0" class="empty">
      <p>No reports yet.</p>
      <button class="primary" @click="showAdd = true">Add your first report</button>
    </div>

    <table v-else class="list">
      <thead>
        <tr>
          <th>Name</th>
          <th>Role</th>
          <th>Cadence</th>
          <th>Started</th>
          <th></th>
        </tr>
      </thead>
      <tbody>
        <tr
          v-for="r in visible"
          :key="r.id"
          :class="{ archived: !r.active }"
          @click="openTimeline(r.id)"
        >
          <td class="name">{{ r.name }}</td>
          <td>{{ r.role ?? "—" }}</td>
          <td>every {{ r.oneOnOneCadenceDays }}d</td>
          <td>{{ r.startDate ?? "—" }}</td>
          <td class="status">
            <span v-if="!r.active" class="badge">archived</span>
          </td>
        </tr>
      </tbody>
    </table>

    <AddReportModal v-if="showAdd" @close="showAdd = false" @created="(id) => openTimeline(id)" />
  </div>
</template>

<style scoped>
.reports-view { max-width: 900px; }
.page-head { display: flex; justify-content: space-between; align-items: center; margin-bottom: 18px; }
h2 { margin: 0; font-size: 20px; }
.actions { display: flex; gap: 12px; align-items: center; }
.archived-toggle { display: inline-flex; gap: 6px; align-items: center; font-size: 12px; color: var(--text-dim); }
.primary {
  background: var(--accent); color: #fff; border: none;
  padding: 7px 14px; border-radius: 4px; font-size: 13px; cursor: pointer;
}
.empty { padding: 48px 0; text-align: center; color: var(--text-dim); }
.empty .primary { margin-top: 12px; }
.list { width: 100%; border-collapse: collapse; font-size: 13px; }
.list th {
  text-align: left; padding: 8px 12px;
  font-size: 10px; text-transform: uppercase; letter-spacing: 0.08em;
  opacity: 0.55; border-bottom: 1px solid var(--border);
}
.list td { padding: 10px 12px; border-bottom: 1px solid var(--border); cursor: pointer; }
.list tr:hover td { background: var(--surface-2); }
.list .name { font-weight: 600; }
.list tr.archived td { opacity: 0.5; }
.badge {
  display: inline-block; padding: 2px 6px; border-radius: 3px;
  background: #374151; font-size: 10px; text-transform: uppercase; letter-spacing: 0.06em;
}
.status { text-align: right; }
</style>
