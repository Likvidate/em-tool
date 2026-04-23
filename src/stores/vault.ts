import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { vaultApi, InvokeError } from "../lib/invoke";

export const useVaultStore = defineStore("vault", () => {
  const exists = ref<boolean | null>(null);
  const unlocked = ref(false);
  const checking = ref(false);
  const lastError = ref<string | null>(null);

  const status = computed<"loading" | "needs-setup" | "locked" | "unlocked">(() => {
    if (exists.value === null) return "loading";
    if (!exists.value) return "needs-setup";
    return unlocked.value ? "unlocked" : "locked";
  });

  async function refresh() {
    checking.value = true;
    try {
      exists.value = await vaultApi.exists();
      unlocked.value = exists.value ? await vaultApi.isUnlocked() : false;
    } finally {
      checking.value = false;
    }
  }

  async function create(password: string) {
    lastError.value = null;
    try {
      await vaultApi.create(password);
      exists.value = true;
      unlocked.value = true;
    } catch (e) {
      lastError.value = e instanceof InvokeError ? e.message : String(e);
      throw e;
    }
  }

  async function unlock(password: string) {
    lastError.value = null;
    try {
      await vaultApi.unlock(password);
      unlocked.value = true;
    } catch (e) {
      lastError.value = e instanceof InvokeError ? e.message : String(e);
      throw e;
    }
  }

  async function lock() {
    await vaultApi.lock();
    unlocked.value = false;
  }

  return { exists, unlocked, checking, lastError, status, refresh, create, unlock, lock };
});
