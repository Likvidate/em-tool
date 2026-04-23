import { ref, computed } from "vue";
import { currentIsoWeek, formatIsoWeek, addWeeks, type IsoWeek } from "../lib/iso-week";

/**
 * Reactive "week cursor" for capture-style screens.
 */
export function useCurrentWeek() {
  const week = ref<IsoWeek>(currentIsoWeek());
  const isoWeek = computed(() => formatIsoWeek(week.value));
  const label = computed(() => `Week ${week.value.week}, ${week.value.year}`);

  function prev() { week.value = addWeeks(week.value, -1); }
  function next() { week.value = addWeeks(week.value, 1); }
  function toCurrent() { week.value = currentIsoWeek(); }
  function set(w: IsoWeek) { week.value = w; }

  return { week, isoWeek, label, prev, next, toCurrent, set };
}
