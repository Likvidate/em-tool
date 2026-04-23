<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useOneOnOnesStore } from "../stores/one-on-ones";
import { useActionItemsStore } from "../stores/action-items";
import { actionItemsApi } from "../lib/invoke";
import type { ActionItemOwner } from "../types/action-item";
import type { OneOnOne } from "../types/one-on-one";

const props = defineProps<{ reportId: number; existing?: OneOnOne }>();
const emit = defineEmits<{ close: []; created: [oneOnOneId: number] }>();

const oneOnOnes = useOneOnOnesStore();
const actionItems = useActionItemsStore();

interface ActionRow {
  // Present on persisted items so we know not to re-create them on save.
  id?: number;
  text: string;
  owner: ActionItemOwner;
  dueDate: string;
  completed: boolean;
  persisted: boolean;
}

function parseWhen(s: string): number | null {
  const t = s.trim();
  const mDateTime = /^(\d{4})-(\d{2})-(\d{2})\s+(\d{2}):(\d{2})$/.exec(t);
  const mDate = /^(\d{4})-(\d{2})-(\d{2})$/.exec(t);
  const m = mDateTime ?? mDate;
  if (!m) return null;
  const y = Number(m[1]), mo = Number(m[2]) - 1, d = Number(m[3]);
  const hh = mDateTime ? Number(m[4]) : 0;
  const mm = mDateTime ? Number(m[5]) : 0;
  const ms = Date.UTC(y, mo, d, hh, mm);
  if (Number.isNaN(ms)) return null;
  return Math.floor(ms / 1000);
}

function defaultWhen(): string {
  const d = new Date();
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${d.getUTCFullYear()}-${pad(d.getUTCMonth() + 1)}-${pad(d.getUTCDate())} ${pad(d.getUTCHours())}:${pad(d.getUTCMinutes())}`;
}

function formatFromUnix(unixSecs: number): string {
  const d = new Date(unixSecs * 1000);
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${d.getUTCFullYear()}-${pad(d.getUTCMonth() + 1)}-${pad(d.getUTCDate())} ${pad(d.getUTCHours())}:${pad(d.getUTCMinutes())}`;
}

function parseDueDate(s: string): string | null {
  const t = s.trim();
  if (!t) return null;
  return /^\d{4}-\d{2}-\d{2}$/.test(t) ? t : null;
}

const isEdit = computed(() => !!props.existing);

const whenStr = ref(props.existing ? formatFromUnix(props.existing.occurredAt) : defaultWhen());
const agenda = ref(props.existing?.agendaMd ?? "");
const notes = ref(props.existing?.notesMd ?? "");
const rows = ref<ActionRow[]>([]);
const submitting = ref(false);
const error = ref<string | null>(null);

onMounted(async () => {
  if (props.existing) {
    try {
      const items = await actionItemsApi.listByMeeting(props.existing.id);
      rows.value = items.map((a) => ({
        id: a.id,
        text: a.text,
        owner: a.owner,
        dueDate: a.dueDate ?? "",
        completed: !!a.completedAt,
        persisted: true,
      }));
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
    }
  }
});

function addRow() {
  rows.value.push({ text: "", owner: "them", dueDate: "", completed: false, persisted: false });
}

async function removeRow(index: number) {
  const row = rows.value[index];
  if (row.persisted && row.id !== undefined) {
    try {
      await actionItems.remove(row.id, props.reportId);
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      return;
    }
  }
  rows.value.splice(index, 1);
}

async function toggleRow(index: number) {
  const row = rows.value[index];
  if (row.persisted && row.id !== undefined) {
    try {
      await actionItems.toggle(row.id, props.reportId);
      row.completed = !row.completed;
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
    }
  } else {
    row.completed = !row.completed;
  }
}

const canSubmit = computed(() => !submitting.value && whenStr.value.trim().length > 0);

async function submit() {
  if (!canSubmit.value) return;
  error.value = null;
  const occurredAt = parseWhen(whenStr.value);
  if (occurredAt === null) {
    error.value = "When must be YYYY-MM-DD HH:MM or YYYY-MM-DD.";
    return;
  }

  // Validate any new rows that have text and a due date.
  const nonEmptyNew = rows.value
    .map((r, i) => ({ r, i }))
    .filter((x) => !x.r.persisted && x.r.text.trim().length > 0);
  for (const { r, i } of nonEmptyNew) {
    if (r.dueDate.trim() && parseDueDate(r.dueDate) === null) {
      error.value = `Action item #${i + 1}: due date must be YYYY-MM-DD.`;
      return;
    }
  }

  submitting.value = true;
  try {
    let targetId: number;
    if (props.existing) {
      const updated = await oneOnOnes.update({
        id: props.existing.id,
        occurredAt,
        agendaMd: agenda.value.trim() || null,
        notesMd: notes.value.trim() || null,
      });
      targetId = updated.id;
    } else {
      const created = await oneOnOnes.create({
        reportId: props.reportId,
        occurredAt,
        agendaMd: agenda.value.trim() || null,
        notesMd: notes.value.trim() || null,
      });
      targetId = created.id;
    }

    for (const { r } of nonEmptyNew) {
      await actionItems.create({
        reportId: props.reportId,
        oneOnOneId: targetId,
        text: r.text.trim(),
        owner: r.owner,
        dueDate: parseDueDate(r.dueDate),
      });
    }

    emit("created", targetId);
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
        <h3>{{ isEdit ? "Edit 1:1" : "Log a 1:1" }}</h3>
        <button class="close" @click="emit('close')">✕</button>
      </header>

      <form @submit.prevent="submit">
        <label><span>When</span>
          <input
            v-model="whenStr"
            type="text"
            autofocus
            placeholder="YYYY-MM-DD HH:MM"
            maxlength="16"
          />
        </label>

        <label><span>Agenda <em class="optional">(optional)</em></span>
          <textarea v-model="agenda" rows="3" placeholder="Topics to cover..."></textarea>
        </label>

        <label><span>Notes / post-meeting reflection <em class="optional">(optional)</em></span>
          <textarea v-model="notes" rows="4" placeholder="What was discussed, how it went..."></textarea>
        </label>

        <div class="action-items">
          <div class="ai-label">Action items</div>
          <div v-if="rows.length === 0" class="empty">No action items yet.</div>
          <div v-for="(row, i) in rows" :key="row.id ?? `new-${i}`" class="ai-row" :class="{ done: row.completed }">
            <input
              type="checkbox"
              class="ai-check"
              :checked="row.completed"
              :disabled="!row.persisted && !row.text.trim()"
              @change="toggleRow(i)"
            />
            <input
              v-model="row.text"
              type="text"
              class="ai-text"
              placeholder="Action item..."
              :disabled="row.persisted"
            />
            <select v-model="row.owner" class="ai-owner" :disabled="row.persisted">
              <option value="them">them</option>
              <option value="me">me</option>
            </select>
            <input
              v-model="row.dueDate"
              type="text"
              class="ai-due"
              placeholder="YYYY-MM-DD"
              maxlength="10"
              :disabled="row.persisted"
            />
            <button type="button" class="ai-remove" @click="removeRow(i)" title="Remove">✕</button>
          </div>
          <button type="button" class="add-row" @click="addRow">+ Add action item</button>
        </div>

        <div v-if="error" class="error">{{ error }}</div>

        <footer>
          <button type="button" class="secondary" @click="emit('close')">Cancel</button>
          <button type="submit" class="primary" :disabled="!canSubmit">
            {{ submitting ? "Saving…" : (isEdit ? "Save changes" : "Save 1:1") }}
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
  border-radius: 8px; max-width: 620px; width: 100%;
  max-height: calc(100vh - 48px); overflow-y: auto;
  box-shadow: 0 30px 80px rgba(0, 0, 0, 0.6);
}
header { display: flex; justify-content: space-between; align-items: center; padding: 16px 18px; border-bottom: 1px solid var(--border); }
header h3 { margin: 0; font-size: 16px; }
.close { background: none; border: none; color: var(--text-dim); font-size: 16px; cursor: pointer; }
form { display: grid; gap: 14px; padding: 18px; }
label { display: grid; gap: 4px; font-size: 12px; color: var(--text-dim); }
input, textarea, select {
  background: var(--bg); border: 1px solid var(--border); color: var(--text);
  padding: 8px 10px; border-radius: 4px; font-family: inherit; font-size: 13px;
}
textarea { resize: vertical; }
.error { color: #f87171; font-size: 12px; }
.optional { font-style: normal; opacity: 0.6; font-weight: 400; }
footer { display: flex; justify-content: flex-end; gap: 8px; margin-top: 4px; }
button { padding: 7px 14px; border: none; border-radius: 4px; font-size: 13px; cursor: pointer; font-family: inherit; }
.primary { background: var(--accent); color: #fff; }
.primary:disabled { opacity: 0.4; cursor: not-allowed; }
.secondary { background: #374151; color: var(--text); }

.action-items { display: grid; gap: 8px; }
.ai-label { font-size: 12px; color: var(--text-dim); }
.empty { font-size: 12px; color: var(--text-dim); opacity: 0.7; font-style: italic; }
.ai-row {
  display: grid;
  grid-template-columns: auto 1fr 90px 130px auto;
  gap: 6px; align-items: center;
}
.ai-check { width: auto; padding: 0; accent-color: var(--accent); cursor: pointer; }
.ai-row.done .ai-text { text-decoration: line-through; opacity: 0.65; }
.ai-row input:disabled, .ai-row select:disabled {
  opacity: 0.75;
  cursor: not-allowed;
}
.ai-remove {
  background: none; border: 1px solid var(--border); color: var(--text-dim);
  padding: 6px 9px; border-radius: 4px; font-size: 12px;
}
.ai-remove:hover { color: var(--red); border-color: var(--red); }
.add-row {
  justify-self: start;
  background: transparent; color: var(--accent);
  border: 1px dashed var(--border);
  padding: 6px 12px; font-size: 12px;
}
.add-row:hover { border-color: var(--accent); }
</style>
