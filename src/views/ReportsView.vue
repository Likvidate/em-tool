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
    <header class="page-header">
      <div>
        <h2 class="page-title">Team members</h2>
        <p class="page-subtitle">Manage the direct reports you're tracking.</p>
      </div>
      <div class="actions">
        <label class="archived-toggle">
          <input v-model="showArchived" type="checkbox" />
          <span>Show archived</span>
        </label>
        <button class="btn btn-primary" @click="showAdd = true">+ Add team member</button>
      </div>
    </header>

    <div v-if="reports.loading && !reports.loaded" class="empty-state">Loading…</div>

    <div v-else-if="visible.length === 0" class="empty-state">
      <p>No team members yet.</p>
      <button class="btn btn-primary" @click="showAdd = true">Add your first team member</button>
    </div>

    <div v-else class="card">
      <table class="data-table">
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
                class="btn btn-ghost btn-sm icon-btn"
                title="Archive (hide, keep history)"
                @click="promptArchive(r.id, r.name, $event)"
              >📦</button>
              <button
                class="btn btn-ghost btn-sm icon-btn danger"
                title="Delete permanently"
                @click="promptDelete(r.id, r.name, $event)"
              >🗑</button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

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
.reports-view {
  max-width: 1000px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: var(--space-5);
}
.actions { display: flex; gap: var(--space-3); align-items: center; }
.archived-toggle {
  display: inline-flex;
  gap: 6px;
  align-items: center;
  font-size: var(--fs-sm);
  color: var(--text-dim);
  cursor: pointer;
  user-select: none;
}
.archived-toggle input { accent-color: var(--accent); }

.data-table td { cursor: pointer; }
.data-table tr.archived td { opacity: 0.5; }
.data-table .name { font-weight: 600; color: var(--text); }

.status { text-align: right; }
.row-actions { text-align: right; white-space: nowrap; width: 88px; }
.icon-btn {
  width: 28px;
  height: 28px;
  padding: 0;
  margin-left: 4px;
}
.icon-btn.danger:hover { color: var(--danger); }
</style>
