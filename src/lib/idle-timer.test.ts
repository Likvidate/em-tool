import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { startIdleTimer } from "./idle-timer";

describe("idle-timer", () => {
  beforeEach(() => { vi.useFakeTimers(); });
  afterEach(() => { vi.useRealTimers(); });

  it("fires onIdle after timeoutMs without activity", () => {
    const onIdle = vi.fn();
    startIdleTimer({ timeoutMs: 1000, onIdle });
    vi.advanceTimersByTime(1001);
    expect(onIdle).toHaveBeenCalledTimes(1);
  });

  it("resets on activity", () => {
    const onIdle = vi.fn();
    startIdleTimer({ timeoutMs: 1000, onIdle });
    vi.advanceTimersByTime(800);
    window.dispatchEvent(new Event("keydown"));
    vi.advanceTimersByTime(800);
    expect(onIdle).not.toHaveBeenCalled();
    vi.advanceTimersByTime(300);
    expect(onIdle).toHaveBeenCalledTimes(1);
  });

  it("stop() cancels the timer", () => {
    const onIdle = vi.fn();
    const stop = startIdleTimer({ timeoutMs: 1000, onIdle });
    stop();
    vi.advanceTimersByTime(2000);
    expect(onIdle).not.toHaveBeenCalled();
  });
});
