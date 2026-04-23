<script setup lang="ts">
import { ref, computed } from "vue";
import { useRouter } from "vue-router";
import { useVaultStore } from "../stores/vault";

const vault = useVaultStore();
const router = useRouter();

const password = ref("");
const confirm = ref("");
const acknowledge = ref(false);
const submitting = ref(false);
const error = ref<string | null>(null);

const passwordsMatch = computed(() => password.value.length > 0 && password.value === confirm.value);
const meetsMinLength = computed(() => password.value.length >= 12);
const canSubmit = computed(() => passwordsMatch.value && meetsMinLength.value && acknowledge.value && !submitting.value);

async function submit() {
  if (!canSubmit.value) return;
  submitting.value = true;
  error.value = null;
  try {
    await vault.create(password.value);
    router.push({ name: "capture" });
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : String(e);
  } finally {
    submitting.value = false;
  }
}
</script>

<template>
  <div class="onboard">
    <div class="card">
      <h1>Welcome to EM Tool</h1>
      <p class="lead">Set a password to encrypt your vault. Everything you write — notes, ratings, plans — lives locally, encrypted with this password.</p>

      <form @submit.prevent="submit">
        <label>
          <span>Password</span>
          <input v-model="password" type="password" autofocus placeholder="At least 12 characters" />
        </label>
        <label>
          <span>Confirm password</span>
          <input v-model="confirm" type="password" placeholder="Re-enter" />
        </label>

        <div v-if="password && !meetsMinLength" class="hint warn">Use at least 12 characters.</div>
        <div v-if="confirm && !passwordsMatch" class="hint warn">Passwords don't match.</div>

        <label class="ack">
          <input v-model="acknowledge" type="checkbox" />
          <span>I understand there is no password recovery. If I forget this password, the vault cannot be opened.</span>
        </label>

        <button type="submit" :disabled="!canSubmit">
          {{ submitting ? "Creating…" : "Create vault" }}
        </button>

        <div v-if="error" class="error">{{ error }}</div>
      </form>
    </div>
  </div>
</template>

<style scoped>
.onboard { display: flex; align-items: center; justify-content: center; min-height: 100vh; padding: 24px; }
.card { background: var(--surface); border: 1px solid var(--border); border-radius: 8px; padding: 28px; max-width: 460px; width: 100%; }
h1 { margin: 0 0 8px; font-size: 22px; }
.lead { color: var(--text-dim); font-size: 13px; line-height: 1.5; margin-bottom: 22px; }
form { display: flex; flex-direction: column; gap: 12px; }
label { display: flex; flex-direction: column; gap: 4px; font-size: 12px; color: var(--text-dim); }
label.ack { flex-direction: row; gap: 8px; align-items: flex-start; margin-top: 6px; color: var(--text); }
label.ack input { margin-top: 2px; }
input[type="password"] {
  background: var(--bg); border: 1px solid var(--border); color: var(--text);
  padding: 8px 10px; border-radius: 4px; font-family: inherit; font-size: 14px;
}
.hint { font-size: 12px; margin-top: -4px; }
.hint.warn { color: #fbbf24; }
button {
  background: var(--accent); color: #fff; border: none; padding: 10px;
  border-radius: 4px; cursor: pointer; font-size: 14px; margin-top: 8px;
}
button:disabled { opacity: 0.4; cursor: not-allowed; }
.error { color: #f87171; font-size: 12px; margin-top: 6px; }
</style>
