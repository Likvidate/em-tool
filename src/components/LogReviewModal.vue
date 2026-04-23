<script setup lang="ts">
import { ref, computed } from "vue";
import { useReviewsStore } from "../stores/reviews";

const props = defineProps<{ reportId: number }>();
const emit = defineEmits<{ close: []; created: [reviewId: number] }>();

const reviews = useReviewsStore();

function parseDate(s: string): number | null {
  const t = s.trim();
  const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(t);
  if (!m) return null;
  const y = Number(m[1]), mo = Number(m[2]) - 1, d = Number(m[3]);
  const ms = Date.UTC(y, mo, d, 0, 0);
  if (Number.isNaN(ms)) return null;
  return Math.floor(ms / 1000);
}

function todayStr(): string {
  const d = new Date();
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${d.getUTCFullYear()}-${pad(d.getUTCMonth() + 1)}-${pad(d.getUTCDate())}`;
}

const period = ref("");
const rating = ref("");
const whenStr = ref(todayStr());
const strengths = ref("");
const devAreas = ref("");
const goals = ref("");
const submitting = ref(false);
const error = ref<string | null>(null);

const canSubmit = computed(() => period.value.trim().length > 0 && !submitting.value);

async function submit() {
  if (!canSubmit.value) return;
  error.value = null;
  const occurredAt = parseDate(whenStr.value);
  if (occurredAt === null) {
    error.value = "When must be YYYY-MM-DD.";
    return;
  }

  submitting.value = true;
  try {
    const created = await reviews.create({
      reportId: props.reportId,
      period: period.value.trim(),
      rating: rating.value.trim() || null,
      strengthsMd: strengths.value.trim() || null,
      devAreasMd: devAreas.value.trim() || null,
      goalsMd: goals.value.trim() || null,
      occurredAt,
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
        <h3>Log a performance review</h3>
        <button class="close" @click="emit('close')">✕</button>
      </header>

      <form @submit.prevent="submit">
        <div class="row">
          <label><span>Period</span>
            <input
              v-model="period"
              type="text"
              autofocus
              placeholder="e.g. Q1 2026"
            />
          </label>
          <label><span>Rating <em class="optional">(optional)</em></span>
            <input
              v-model="rating"
              type="text"
              placeholder="e.g. Exceeds"
            />
          </label>
        </div>

        <label><span>When</span>
          <input
            v-model="whenStr"
            type="text"
            placeholder="YYYY-MM-DD"
            pattern="\d{4}-\d{2}-\d{2}"
            inputmode="numeric"
            maxlength="10"
          />
        </label>

        <label><span>Strengths</span>
          <textarea v-model="strengths" rows="3" placeholder="What they do well..."></textarea>
        </label>

        <label><span>Development areas</span>
          <textarea v-model="devAreas" rows="3" placeholder="Growth opportunities..."></textarea>
        </label>

        <label><span>Goals for next cycle</span>
          <textarea v-model="goals" rows="3" placeholder="What to focus on next..."></textarea>
        </label>

        <div v-if="error" class="error">{{ error }}</div>

        <footer>
          <button type="button" class="secondary" @click="emit('close')">Cancel</button>
          <button type="submit" class="primary" :disabled="!canSubmit">
            {{ submitting ? "Saving…" : "Save review" }}
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
  border-radius: 8px; max-width: 560px; width: 100%;
  max-height: calc(100vh - 48px); overflow-y: auto;
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
.optional { font-style: normal; opacity: 0.6; font-weight: 400; }
footer { display: flex; justify-content: flex-end; gap: 8px; margin-top: 4px; }
button { padding: 7px 14px; border: none; border-radius: 4px; font-size: 13px; cursor: pointer; font-family: inherit; }
.primary { background: var(--accent); color: #fff; }
.primary:disabled { opacity: 0.4; cursor: not-allowed; }
.secondary { background: #374151; color: var(--text); }
</style>
