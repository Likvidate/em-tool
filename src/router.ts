import { createRouter, createWebHistory, type RouteRecordRaw } from "vue-router";
import { useVaultStore } from "./stores/vault";

const routes: RouteRecordRaw[] = [
  { path: "/", redirect: "/capture" },
  { path: "/onboard", name: "onboard", component: () => import("./views/OnboardingView.vue") },
  { path: "/unlock", name: "unlock", component: () => import("./views/UnlockView.vue") },
  { path: "/capture", name: "capture", component: () => import("./views/WeeklyCaptureView.vue") },
  { path: "/reports", name: "reports", component: () => import("./views/ReportsView.vue") },
  { path: "/reports/:id/timeline", name: "report-timeline",
    component: () => import("./views/ReportTimelineView.vue") },
  { path: "/heatmap", name: "heatmap", component: () => import("./views/TeamHeatmapView.vue") },
  { path: "/plan", name: "plan", component: () => import("./views/PlanGeneratorView.vue") },
  { path: "/settings", name: "settings", component: () => import("./views/SettingsView.vue") },
];

const router = createRouter({ history: createWebHistory(), routes });

router.beforeEach(async (to) => {
  const vault = useVaultStore();
  if (vault.status === "loading") {
    await vault.refresh();
  }

  if (vault.status === "needs-setup" && to.name !== "onboard") {
    return { name: "onboard" };
  }
  if (vault.status === "locked" && to.name !== "unlock") {
    return { name: "unlock" };
  }
  if (vault.status === "unlocked" && (to.name === "onboard" || to.name === "unlock")) {
    return { name: "capture" };
  }
});

export default router;
