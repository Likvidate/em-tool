<script setup lang="ts">
import { RouterView, useRoute } from "vue-router";
import AppSidebar from "./components/AppSidebar.vue";
import { computed, onMounted, onUnmounted, watch } from "vue";
import { useVaultStore } from "./stores/vault";
import { startIdleTimer } from "./lib/idle-timer";
import { vaultApi } from "./lib/invoke";

const vault = useVaultStore();
const route = useRoute();
const showChrome = computed(() => !["onboard", "unlock"].includes(String(route.name)));

const AUTO_LOCK_MS = 15 * 60 * 1000;

let stop: (() => void) | null = null;

function start() {
  if (stop) stop();
  stop = startIdleTimer({
    timeoutMs: AUTO_LOCK_MS,
    onIdle: () => { vault.lock(); },
    onActivity: () => { vaultApi.touchActivity(); },
  });
}

onMounted(async () => {
  await vault.refresh();
  if (vault.unlocked) start();
});

watch(() => vault.unlocked, (now) => {
  if (now) start();
  else if (stop) { stop(); stop = null; }
});

onUnmounted(() => { if (stop) stop(); });
</script>

<template>
  <div class="app">
    <AppSidebar v-if="showChrome" />
    <main class="main">
      <RouterView v-slot="{ Component }">
        <Transition name="fade" mode="out-in">
          <component :is="Component" />
        </Transition>
      </RouterView>
    </main>
  </div>
</template>

<style scoped>
.app { display: flex; height: 100vh; }
.main {
  flex: 1;
  overflow: auto;
  padding: var(--space-6) var(--space-6) var(--space-8);
}
</style>
