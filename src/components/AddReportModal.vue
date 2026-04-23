<script setup lang="ts">
import { ref, computed } from "vue";
import { useReportsStore } from "../stores/reports";

const emit = defineEmits<{ close: []; created: [id: number] }>();
const reports = useReportsStore();

const name = ref("");
const role = ref("");
const startDate = ref("");
const cadence = ref(14);
const notes = ref("");
const submitting = ref(false);
const error = ref<string | null>(null);

const canSubmit = computed(() => name.value.trim().length > 0 && !submitting.value);

async function submit() {
  if (!canSubmit.value) return;
  submitting.value = true;
  error.value = null;
  try {
    const created = await reports.create({
      name: name.value.trim(),
      role: role.value.trim() || null,
      startDate: startDate.value || null,
      oneOnOneCadenceDays: cadence.value,
      notes: notes.value.trim() || null,
    });
    emit("created", created.id);
    emit("close");
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : String(e);
  } finally {
    submitting.value = false;
  }
}
</script>

<template>
  <div class="backdrop" @click.self="emit('close')">
    <div class="modal">
      <header>
        <h3>Add a team member</h3>
        <button class="close" @click="emit('close')">✕</button>
      </header>

      <form @submit.prevent="submit">
        <label><span>Name</span>
          <input v-model="name" type="text" autofocus placeholder="e.g. Fatima Al-Sayed" />
        </label>
        <div class="row">
          <label><span>Role</span>
            <input v-model="role" type="text" placeholder="e.g. Senior Backend" />
          </label>
          <label><span>Start date on team</span>
            <input v-model="startDate" type="date" />
          </label>
        </div>
        <label><span>1:1 cadence</span>
          <select v-model.number="cadence">
            <option :value="7">Weekly</option>
            <option :value="14">Bi-weekly (every 2 weeks)</option>
            <option :value="21">Every 3 weeks</option>
            <option :value="30">Monthly</option>
            <option :value="45">Every 6 weeks</option>
            <option :value="60">Every 2 months</option>
            <option :value="90">Quarterly (every 3 months)</option>
            <option :value="120">Every 4 months</option>
            <option :value="180">Every 6 months</option>
          </select>
        </label>
        <label><span>Notes</span>
          <textarea v-model="notes" rows="3" placeholder="Anything you want to remember..."></textarea>
        </label>

        <div v-if="error" class="error">{{ error }}</div>

        <footer>
          <button type="button" class="secondary" @click="emit('close')">Cancel</button>
          <button type="submit" class="primary" :disabled="!canSubmit">
            {{ submitting ? "Adding…" : "Add team member" }}
          </button>
        </footer>
      </form>
    </div>
  </div>
</template>

<style scoped>
.backdrop {
  position: fixed; inset: 0; z-index: 100;
  background: rgba(0, 0, 0, 0.55);
  display: flex; align-items: center; justify-content: center; padding: 24px;
}
.modal {
  background: var(--surface); border: 1px solid var(--border);
  border-radius: 8px; max-width: 520px; width: 100%;
  box-shadow: 0 30px 80px rgba(0, 0, 0, 0.6);
}
header { display: flex; justify-content: space-between; align-items: center; padding: 16px 18px; border-bottom: 1px solid var(--border); }
header h3 { margin: 0; font-size: 16px; }
.close { background: none; border: none; color: var(--text-dim); font-size: 16px; cursor: pointer; }
form { display: grid; gap: 14px; padding: 18px; }
.row { display: grid; grid-template-columns: 1fr 1fr; gap: 14px; }
label { display: grid; gap: 4px; font-size: 12px; color: var(--text-dim); }
input, textarea, select {
  background: var(--bg); border: 1px solid var(--border); color: var(--text);
  padding: 8px 10px; border-radius: 4px; font-family: inherit; font-size: 13px;
}
textarea { resize: vertical; }
.error { color: #f87171; font-size: 12px; }
footer { display: flex; justify-content: flex-end; gap: 8px; margin-top: 4px; }
button { padding: 7px 14px; border: none; border-radius: 4px; font-size: 13px; cursor: pointer; }
.primary { background: var(--accent); color: #fff; }
.primary:disabled { opacity: 0.4; cursor: not-allowed; }
.secondary { background: #374151; color: var(--text); }
</style>
