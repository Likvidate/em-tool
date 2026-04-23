<script setup lang="ts">
import type { Color } from "../lib/colors";

defineProps<{
  /** Cells in chronological order. `null` renders as empty. */
  cells: { isoWeek: string; color: Color | null; title?: string }[];
}>();
</script>

<template>
  <div class="strip">
    <div
      v-for="c in cells"
      :key="c.isoWeek"
      class="cell"
      :class="c.color ?? 'none'"
      :title="c.title ?? c.isoWeek"
    />
  </div>
</template>

<style scoped>
.strip { display: flex; gap: 2px; padding: 12px 14px; background: #141414; border: 1px solid var(--border); border-top: none; }
.cell { flex: 1; min-width: 6px; height: 22px; border-radius: 2px; background: #222; }
.cell.red    { background: #ef4444; }
.cell.yellow { background: #facc15; }
.cell.grey   { background: #6b7280; }
.cell.green  { background: #4ade80; }
.cell.blue   { background: #3b82f6; }
.cell.none   { background: #1a1a1a; }
</style>
