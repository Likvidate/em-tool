<script setup lang="ts">
defineProps<{
  label: string;
  isCurrent?: boolean;
}>();
defineEmits<{ prev: []; next: []; jumpToCurrent: [] }>();
</script>

<template>
  <div class="weeknav">
    <button @click="$emit('prev')" aria-label="Previous week">◀</button>
    <div class="label">
      <span>{{ label }}</span>
      <button v-if="!isCurrent" class="today" @click="$emit('jumpToCurrent')">Today</button>
    </div>
    <button @click="$emit('next')" aria-label="Next week">▶</button>
  </div>
</template>

<style scoped>
.weeknav {
  display: flex; align-items: center; gap: 12px;
  padding: 8px 12px;
  background: var(--surface-2); border: 1px solid var(--border);
  border-radius: 6px;
}
.weeknav > button {
  background: none; border: 1px solid var(--border); color: var(--text);
  width: 28px; height: 28px; border-radius: 4px; cursor: pointer;
  font-size: 11px;
}
.label { flex: 1; display: flex; justify-content: center; align-items: center; gap: 10px; font-weight: 600; }
.today {
  background: none; border: 1px solid var(--border); color: var(--text-dim);
  padding: 2px 8px; border-radius: 3px; font-size: 11px; cursor: pointer;
}
</style>
