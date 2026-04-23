<script setup lang="ts">
import { RouterLink } from "vue-router";

const items = [
  { to: "/capture", label: "Weekly capture", icon: "📋" },
  { to: "/reports", label: "Team members", icon: "👤" },
  { to: "/heatmap", label: "Team heatmap", icon: "🔥" },
  { to: "/plan", label: "Plan generator", icon: "🎯" },
  { to: "/settings", label: "Settings", icon: "⚙" },
];
</script>

<template>
  <aside class="sidebar">
    <div class="brand">EM Tool</div>
    <nav>
      <RouterLink
        v-for="item in items"
        :key="item.to"
        :to="item.to"
        class="nav-item"
        active-class="active"
      >
        <span class="icon">{{ item.icon }}</span>
        <span>{{ item.label }}</span>
      </RouterLink>
    </nav>
  </aside>
</template>

<style scoped>
.sidebar {
  width: 220px;
  background: var(--surface);
  border-right: 1px solid var(--border);
  padding: 18px 12px;
  display: flex;
  flex-direction: column;
  gap: 20px;
}
.brand {
  font-weight: 700;
  font-size: 16px;
  padding: 0 8px;
}
nav { display: flex; flex-direction: column; gap: 2px; }
.nav-item {
  display: flex;
  gap: 10px;
  padding: 8px 10px;
  border-radius: 5px;
  text-decoration: none;
  color: var(--text-dim);
  font-size: 13px;
}
.nav-item:hover { background: var(--surface-2); color: var(--text); }
.nav-item.active { background: var(--surface-2); color: var(--text); }
.icon { width: 18px; }
</style>
