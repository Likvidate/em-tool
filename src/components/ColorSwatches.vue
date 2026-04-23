<script setup lang="ts">
import { COLOR_ORDER, COLORS, type Color } from "../lib/colors";

const props = defineProps<{ modelValue: Color | null }>();
const emit = defineEmits<{ "update:modelValue": [value: Color | null] }>();

function pick(color: Color) {
  emit("update:modelValue", props.modelValue === color ? null : color);
}
</script>

<template>
  <div class="swatches" role="radiogroup" aria-label="Color rating">
    <button
      v-for="c in COLOR_ORDER"
      :key="c"
      type="button"
      class="sw"
      :class="[c, { active: modelValue === c }]"
      :aria-label="COLORS[c].label"
      :title="COLORS[c].description"
      role="radio"
      :aria-checked="modelValue === c"
      @click="pick(c)"
    />
  </div>
</template>

<style scoped>
.swatches { display: inline-flex; gap: 5px; }
.sw {
  width: 22px; height: 22px;
  border-radius: 4px;
  cursor: pointer;
  border: 2px solid transparent;
  padding: 0;
}
.sw.red { background: #ef4444; }
.sw.yellow { background: #facc15; }
.sw.grey { background: #6b7280; }
.sw.green { background: #4ade80; }
.sw.blue { background: #3b82f6; }
.sw.active { border-color: #fff; box-shadow: 0 0 0 2px rgba(255, 255, 255, 0.2); }
.sw:focus-visible { outline: 2px solid #7c3aed; outline-offset: 2px; }
</style>
