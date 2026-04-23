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
    <div class="brand">
      <div class="logo">EM</div>
      <div class="brand-text">
        <div class="name">EM Tool</div>
        <div class="tag">Local · Private</div>
      </div>
    </div>

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

    <div class="sidebar-foot">
      <div class="foot-line">v0.1.0</div>
    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  width: 240px;
  flex-shrink: 0;
  background: linear-gradient(180deg, var(--surface) 0%, var(--bg-2) 100%);
  border-right: 1px solid var(--border);
  padding: var(--space-5) var(--space-3);
  display: flex;
  flex-direction: column;
  gap: var(--space-6);
}

.brand {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: 0 var(--space-2);
}
.logo {
  width: 32px; height: 32px;
  border-radius: var(--radius-md);
  background: linear-gradient(135deg, var(--accent) 0%, #6366f1 100%);
  color: #fff;
  font-weight: 700;
  font-size: 13px;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 4px 14px rgba(139, 92, 246, 0.35);
  letter-spacing: -0.02em;
}
.brand-text { display: flex; flex-direction: column; line-height: 1.15; }
.brand-text .name { font-weight: 700; font-size: var(--fs-md); letter-spacing: -0.01em; }
.brand-text .tag { font-size: var(--fs-xs); color: var(--text-mute); font-weight: 500; }

nav { display: flex; flex-direction: column; gap: 2px; flex: 1; }
.nav-item {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: 9px var(--space-3);
  border-radius: var(--radius-md);
  text-decoration: none;
  color: var(--text-dim);
  font-size: var(--fs-base);
  font-weight: 500;
  transition: background var(--t-fast), color var(--t-fast);
  position: relative;
}
.nav-item:hover {
  background: var(--surface-2);
  color: var(--text);
}
.nav-item.active {
  background: var(--accent-dim);
  color: var(--text);
  font-weight: 600;
}
.nav-item.active::before {
  content: "";
  position: absolute;
  left: -12px;
  top: 6px;
  bottom: 6px;
  width: 3px;
  background: var(--accent);
  border-radius: 3px;
}
.icon { width: 18px; text-align: center; font-size: 14px; }

.sidebar-foot {
  padding: var(--space-3) var(--space-3) 0;
  border-top: 1px solid var(--border);
  margin-top: auto;
}
.foot-line { font-size: var(--fs-xs); color: var(--text-mute); }
</style>
