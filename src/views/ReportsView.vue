<script setup lang="ts">
import { onMounted, ref, computed } from "vue";
import { useRouter } from "vue-router";
import { useReportsStore } from "../stores/reports";
import AddReportModal from "../components/AddReportModal.vue";
import ConfirmModal from "../components/ConfirmModal.vue";

const reports = useReportsStore();
const router = useRouter();
const showAdd = ref(false);
const showArchived = ref(false);

type PendingAction = { kind: "archive" | "delete"; id: number; name: string };
const pending = ref<PendingAction | null>(null);

const visible = computed(() =>
  showArchived.value ? reports.items : reports.active,
);

onMounted(() => {
  if (!reports.loaded) reports.load(true);
});

function openTimeline(id: number) {
  router.push({ name: "report-timeline", params: { id: String(id) } });
}

function promptArchive(id: number, name: string, ev: Event) {
  ev.stopPropagation();
  pending.value = { kind: "archive", id, name };
}

function promptDelete(id: number, name: string, ev: Event) {
  ev.stopPropagation();
  pending.value = { kind: "delete", id, name };
}

async function confirmPending() {
  if (!pending.value) return;
  const { kind, id } = pending.value;
  pending.value = null;
  if (kind === "archive") await reports.archive(id);
  else await reports.remove(id);
}
</script>

<template>
  <div class="reports-view">
    <header class="page-head">
      <h2>Team members</h2>
      <div class="actions">
        <label class="archived-toggle">
          <input v-model="showArchived" type="checkbox" />
          <span>Show archived</span>
        </label>
        <button class="primary" @click="showAdd = true">+ Add team member</button>
      </div>
    </header>

    <div v-if="reports.loading && !reports.loaded" class="empty">Loading…</div>

    <div v-else-if="visible.length === 0" class="empty">
      <p>No team members yet.</p>
      <button class="primary" @click="showAdd = true">Add your first team member</button>
    </div>

    <table v-else class="list">
      <thead>
        <tr>
          <th>Name</th>
          <th>Role</th>
          <th>Cadence</th>
          <th>Started</th>
          <th></th>
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
          <td class="row-actions">
            <button
              v-if="r.active"
              class="icon-btn"
              title="Archive (hide, keep history)"
              @click="promptArchive(r.id, r.name, $event)"
            >📦</button>
            <button
              class="icon-btn danger"
              title="Delete permanently"
              @click="promptDelete(r.id, r.name, $event)"
            >🗑</button>
          </td>
        </tr>
      </tbody>
    </table>

    <AddReportModal v-if="showAdd" @close="showAdd = false" @created="(id) => openTimeline(id)" />

    <ConfirmModal
      v-if="pending?.kind === 'archive'"
      title="Archive team member?"
      :message="`${pending.name} will be hidden from the capture grid and heatmap, but their history stays. You can restore them anytime by enabling 'Show archived'.`"
      confirm-label="Archive"
      @confirm="confirmPending"
      @cancel="pending = null"
    />

    <ConfirmModal
      v-if="pending?.kind === 'delete'"
      title="Delete permanently?"
      :message="`${pending.name} and all their ratings, 1:1 notes, action items, and reviews will be permanently deleted. This can't be undone.`"
      confirm-label="Delete forever"
      variant="danger"
      @confirm="confirmPending"
      @cancel="pending = null"
    />
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
.row-actions { text-align: right; white-space: nowrap; width: 72px; }
.icon-btn {
  background: none; border: 1px solid var(--border); color: var(--text-dim);
  width: 28px; height: 28px; border-radius: 4px; cursor: pointer;
  font-size: 13px; margin-left: 4px; padding: 0;
}
.icon-btn:hover { color: var(--text); background: var(--surface-2); }
.icon-btn.danger:hover { color: #f87171; border-color: #b91c1c; }
</style>
