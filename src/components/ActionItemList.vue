<script setup lang="ts">
import { computed } from "vue";
import type { ActionItem } from "../types/action-item";

const props = withDefaults(
  defineProps<{
    items: ActionItem[];
    showCompleted?: boolean;
  }>(),
  { showCompleted: false },
);

const emit = defineEmits<{
  toggle: [id: number];
  delete: [id: number];
}>();

const visible = computed(() =>
  props.showCompleted ? props.items : props.items.filter((a) => !a.completedAt),
);

function todayStr(): string {
  const d = new Date();
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${d.getUTCFullYear()}-${pad(d.getUTCMonth() + 1)}-${pad(d.getUTCDate())}`;
}
const today = todayStr();

function isOverdue(a: ActionItem): boolean {
  return a.dueDate !== null && !a.completedAt && a.dueDate < today;
}
</script>

<template>
  <div class="list">
    <div v-if="visible.length === 0" class="empty">No action items.</div>
    <div
      v-for="item in visible"
      :key="item.id"
      class="row"
      :class="{ done: !!item.completedAt }"
    >
      <input
        type="checkbox"
        :checked="!!item.completedAt"
        @change="emit('toggle', item.id)"
      />
      <span class="text">{{ item.text }}</span>
      <span class="badge" :class="item.owner">{{ item.owner }}</span>
      <span
        v-if="item.dueDate"
        class="due"
        :class="{ overdue: isOverdue(item) }"
      >{{ item.dueDate }}</span>
      <button
        class="delete"
        title="Delete"
        @click="emit('delete', item.id)"
      >✕</button>
    </div>
  </div>
</template>

<style scoped>
.list {
  display: flex; flex-direction: column; gap: 4px;
}
.empty {
  text-align: center;
  color: var(--text-dim);
  font-size: 12px;
  font-style: italic;
  padding: 16px 0;
  opacity: 0.7;
}
.row {
  display: grid;
  grid-template-columns: auto 1fr auto auto auto;
  gap: 10px;
  align-items: center;
  padding: 7px 4px;
  border-bottom: 1px solid var(--border);
  font-size: 13px;
}
.row:last-child { border-bottom: none; }
.row .delete {
  opacity: 0;
  background: none;
  border: none;
  color: var(--text-dim);
  font-size: 12px;
  cursor: pointer;
  padding: 2px 6px;
  border-radius: 3px;
  transition: color 120ms, background 120ms;
}
.row:hover .delete { opacity: 1; }
.row .delete:hover { color: var(--red); background: rgba(239, 68, 68, 0.08); }

input[type="checkbox"] {
  accent-color: var(--accent);
  cursor: pointer;
}
.text {
  color: var(--text);
  line-height: 1.4;
  word-break: break-word;
}
.row.done .text {
  text-decoration: line-through;
  color: var(--text-dim);
  opacity: 0.65;
}
.badge {
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  padding: 2px 6px;
  border-radius: 3px;
  font-weight: 600;
  background: var(--surface-2);
  color: var(--text-dim);
  border: 1px solid var(--border);
}
.badge.me { color: var(--accent); border-color: color-mix(in srgb, var(--accent) 40%, var(--border)); }
.badge.them { color: var(--text-dim); }
.due {
  font-size: 11px;
  color: var(--text-dim);
  white-space: nowrap;
  font-variant-numeric: tabular-nums;
}
.due.overdue { color: var(--red); font-weight: 500; }
.row.done .due { text-decoration: line-through; opacity: 0.6; }
</style>
