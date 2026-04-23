import { createRouter, createWebHistory, type RouteRecordRaw } from "vue-router";

const routes: RouteRecordRaw[] = [
  { path: "/", redirect: "/capture" },
  { path: "/onboard", name: "onboard", component: () => import("./views/OnboardingView.vue") },
  { path: "/unlock", name: "unlock", component: () => import("./views/UnlockView.vue") },
  { path: "/capture", name: "capture", component: () => import("./views/WeeklyCaptureView.vue") },
  { path: "/reports", name: "reports", component: () => import("./views/ReportsView.vue") },
  { path: "/heatmap", name: "heatmap", component: () => import("./views/TeamHeatmapView.vue") },
  { path: "/plan", name: "plan", component: () => import("./views/PlanGeneratorView.vue") },
  { path: "/settings", name: "settings", component: () => import("./views/SettingsView.vue") },
];

export default createRouter({ history: createWebHistory(), routes });
