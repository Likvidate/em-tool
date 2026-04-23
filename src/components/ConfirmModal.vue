<script setup lang="ts">
withDefaults(
  defineProps<{
    title: string;
    message: string;
    confirmLabel?: string;
    cancelLabel?: string;
    variant?: "danger" | "default";
  }>(),
  { confirmLabel: "Confirm", cancelLabel: "Cancel", variant: "default" },
);
defineEmits<{ confirm: []; cancel: [] }>();
</script>

<template>
  <div class="backdrop" @click.self="$emit('cancel')">
    <div class="modal" :class="variant">
      <div class="icon">
        <span v-if="variant === 'danger'">⚠</span>
        <span v-else>?</span>
      </div>
      <h3>{{ title }}</h3>
      <p>{{ message }}</p>
      <footer>
        <button class="secondary" @click="$emit('cancel')">{{ cancelLabel }}</button>
        <button :class="variant === 'danger' ? 'danger' : 'primary'" @click="$emit('confirm')" autofocus>
          {{ confirmLabel }}
        </button>
      </footer>
    </div>
  </div>
</template>

<style scoped>
.backdrop {
  position: fixed; inset: 0; z-index: 200;
  background: rgba(0, 0, 0, 0.6);
  display: flex; align-items: center; justify-content: center; padding: 24px;
  animation: fadeIn 120ms ease-out;
}
.modal {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 10px;
  max-width: 420px; width: 100%;
  padding: 24px;
  box-shadow: 0 30px 80px rgba(0, 0, 0, 0.6);
  animation: slideIn 160ms ease-out;
}
.icon {
  width: 44px; height: 44px; border-radius: 50%;
  display: flex; align-items: center; justify-content: center;
  font-size: 22px;
  background: #374151;
  color: #9ca3af;
  margin-bottom: 12px;
}
.modal.danger .icon { background: rgba(239, 68, 68, 0.15); color: #f87171; }
h3 { margin: 0 0 6px; font-size: 16px; }
p { margin: 0 0 18px; font-size: 13px; line-height: 1.55; color: var(--text-dim); }
footer { display: flex; justify-content: flex-end; gap: 8px; }
button { padding: 7px 14px; border: none; border-radius: 5px; font-size: 13px; cursor: pointer; font-family: inherit; }
.primary { background: var(--accent); color: #fff; }
.danger { background: #dc2626; color: #fff; }
.danger:hover { background: #b91c1c; }
.secondary { background: #374151; color: var(--text); }
.secondary:hover { background: #4b5563; }

@keyframes fadeIn { from { opacity: 0; } to { opacity: 1; } }
@keyframes slideIn {
  from { opacity: 0; transform: translateY(8px) scale(0.97); }
  to   { opacity: 1; transform: translateY(0) scale(1); }
}
</style>
