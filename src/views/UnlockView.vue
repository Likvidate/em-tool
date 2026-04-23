<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useRouter } from "vue-router";
import { useVaultStore } from "../stores/vault";

const vault = useVaultStore();
const router = useRouter();

const password = ref("");
const submitting = ref(false);
const error = ref<string | null>(null);
const failureCount = ref(0);
const cooldownUntil = ref<number | null>(null);
const now = ref(Date.now());

const inCooldown = computed(() => cooldownUntil.value !== null && now.value < cooldownUntil.value);
const cooldownSecs = computed(() =>
  cooldownUntil.value ? Math.max(0, Math.ceil((cooldownUntil.value - now.value) / 1000)) : 0
);

onMounted(() => {
  setInterval(() => { now.value = Date.now(); }, 500);
});

async function submit() {
  if (inCooldown.value) return;
  submitting.value = true;
  error.value = null;
  try {
    await vault.unlock(password.value);
    password.value = "";
    failureCount.value = 0;
    cooldownUntil.value = null;
    router.push({ name: "capture" });
  } catch (e: unknown) {
    failureCount.value += 1;
    if (failureCount.value >= 5) {
      cooldownUntil.value = Date.now() + 60_000;
      failureCount.value = 0;
    }
    error.value = "Wrong password.";
  } finally {
    submitting.value = false;
  }
}
</script>

<template>
  <div class="unlock">
    <div class="card">
      <h1>Unlock your vault</h1>

      <form @submit.prevent="submit">
        <label>
          <span>Password</span>
          <input v-model="password" type="password" autofocus :disabled="inCooldown" />
        </label>

        <button type="submit" :disabled="inCooldown || submitting || !password">
          <template v-if="inCooldown">Too many attempts — try again in {{ cooldownSecs }}s</template>
          <template v-else>{{ submitting ? "Unlocking…" : "Unlock" }}</template>
        </button>

        <div v-if="error && !inCooldown" class="error">{{ error }}</div>
      </form>
    </div>
  </div>
</template>

<style scoped>
.unlock { display: flex; align-items: center; justify-content: center; min-height: 100vh; padding: 24px; }
.card { background: var(--surface); border: 1px solid var(--border); border-radius: 8px; padding: 28px; max-width: 420px; width: 100%; }
h1 { margin: 0 0 18px; font-size: 20px; }
form { display: flex; flex-direction: column; gap: 12px; }
label { display: flex; flex-direction: column; gap: 4px; font-size: 12px; color: var(--text-dim); }
input[type="password"] {
  background: var(--bg); border: 1px solid var(--border); color: var(--text);
  padding: 8px 10px; border-radius: 4px; font-family: inherit; font-size: 14px;
}
button {
  background: var(--accent); color: #fff; border: none; padding: 10px;
  border-radius: 4px; cursor: pointer; font-size: 14px; margin-top: 8px;
}
button:disabled { opacity: 0.4; cursor: not-allowed; }
.error { color: #f87171; font-size: 12px; }
</style>
