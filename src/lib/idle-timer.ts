export interface IdleTimerOptions {
  timeoutMs: number;
  onIdle: () => void;
  onActivity?: () => void;
}

export function startIdleTimer({ timeoutMs, onIdle, onActivity }: IdleTimerOptions) {
  let timer: number | null = null;
  let active = true;

  const events = ["mousemove", "keydown", "scroll", "click", "touchstart"];

  const reset = () => {
    if (!active) return;
    if (timer !== null) clearTimeout(timer);
    if (onActivity) onActivity();
    timer = window.setTimeout(() => {
      if (active) onIdle();
    }, timeoutMs);
  };

  events.forEach((ev) => window.addEventListener(ev, reset, { passive: true }));
  reset();

  return () => {
    active = false;
    if (timer !== null) clearTimeout(timer);
    events.forEach((ev) => window.removeEventListener(ev, reset));
  };
}
