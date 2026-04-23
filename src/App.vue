<script setup lang="ts">
import { RouterView, useRoute } from "vue-router";
import AppSidebar from "./components/AppSidebar.vue";
import { computed } from "vue";

const route = useRoute();
const showChrome = computed(() => !["onboard", "unlock"].includes(String(route.name)));
</script>

<template>
  <div class="app">
    <AppSidebar v-if="showChrome" />
    <main class="main">
      <RouterView />
    </main>
  </div>
</template>

<style scoped>
.app { display: flex; height: 100vh; }
.main { flex: 1; overflow: auto; padding: 24px; }
</style>
