<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useReportsStore } from "../stores/reports";
import type { Report } from "../types/report";

const props = defineProps<{ existing?: Report }>();
const emit = defineEmits<{ close: []; saved: [id: number] }>();
const reports = useReportsStore();

const isEdit = computed(() => !!props.existing);

const name = ref("");
const role = ref("");
const startDate = ref("");
const cadence = ref(14);
const notes = ref("");
const submitting = ref(false);
const error = ref<string | null>(null);

onMounted(() => {
  if (props.existing) {
    name.value = props.existing.name;
    role.value = props.existing.role ?? "";
    startDate.value = props.existing.startDate ?? "";
    cadence.value = props.existing.oneOnOneCadenceDays;
    notes.value = props.existing.notes ?? "";
  }
});

const canSubmit = computed(() => name.value.trim().length > 0 && !submitting.value);

async function submit() {
  if (!canSubmit.value) return;
  submitting.value = true;
  error.value = null;
  try {
    if (props.existing) {
      const updated = await reports.update({
        id: props.existing.id,
        name: name.value.trim(),
        role: role.value.trim() || null,
        startDate: startDate.value || null,
        oneOnOneCadenceDays: cadence.value,
        notes: notes.value.trim() || null,
      });
      emit("saved", updated.id);
    } else {
      const created = await reports.create({
        name: name.value.trim(),
        role: role.value.trim() || null,
        startDate: startDate.value || null,
        oneOnOneCadenceDays: cadence.value,
        notes: notes.value.trim() || null,
      });
      emit("saved", created.id);
    }
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
        <h3>{{ isEdit ? "Edit team member" : "Add a team member" }}</h3>
        <button class="close" @click="emit('close')">✕</button>
      </header>

      <form @submit.prevent="submit">
        <label><span>Name</span>
          <input v-model="name" type="text" autofocus placeholder="e.g. Fatima Al-Sayed" class="field-input" />
        </label>
        <div class="row">
          <label><span>Role</span>
            <input v-model="role" type="text" placeholder="e.g. Senior Backend" class="field-input" />
          </label>
          <label><span>Start date on team <em class="optional">(optional)</em></span>
            <input
              v-model="startDate"
              type="text"
              placeholder="YYYY-MM-DD"
              pattern="\d{4}-\d{2}-\d{2}"
              inputmode="numeric"
              maxlength="10"
              class="field-input"
            />
          </label>
        </div>
        <label><span>1:1 cadence</span>
          <select v-model.number="cadence" class="field-input">
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
          <textarea v-model="notes" rows="3" placeholder="Anything you want to remember..." class="field-input"></textarea>
        </label>

        <div v-if="error" class="error">{{ error }}</div>

        <footer>
          <button type="button" class="btn btn-secondary" @click="emit('close')">Cancel</button>
          <button type="submit" class="btn btn-primary" :disabled="!canSubmit">
            {{ submitting
              ? (isEdit ? "Saving…" : "Adding…")
              : (isEdit ? "Save changes" : "Add team member") }}
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
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-xl);
  max-width: 520px; width: 100%;
  box-shadow: var(--shadow-lg);
}
header {
  display: flex; justify-content: space-between; align-items: center;
  padding: var(--space-4) var(--space-5);
  border-bottom: 1px solid var(--border);
}
header h3 { margin: 0; font-size: var(--fs-lg); }
.close {
  background: none; border: none; color: var(--text-dim);
  font-size: 16px; cursor: pointer; padding: 4px 8px;
  border-radius: var(--radius-sm);
}
.close:hover { color: var(--text); background: var(--surface-2); }
form { display: grid; gap: var(--space-4); padding: var(--space-5); }
.row { display: grid; grid-template-columns: 1fr 1fr; gap: var(--space-4); }
label {
  display: grid; gap: 4px;
  font-size: var(--fs-xs);
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--text-mute);
  font-weight: 600;
}
label span { display: flex; align-items: center; gap: 6px; }
textarea.field-input { resize: vertical; min-height: 70px; }
.error { color: var(--danger); font-size: var(--fs-sm); }
.optional { font-style: normal; opacity: 0.65; font-weight: 400; text-transform: none; letter-spacing: 0; }
footer { display: flex; justify-content: flex-end; gap: var(--space-2); margin-top: var(--space-1); }
</style>
