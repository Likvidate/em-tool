import { describe, it, expect } from "vitest";
import { currentIsoWeek, parseIsoWeek, formatIsoWeek, addWeeks, weeksInRange } from "./iso-week";

describe("iso-week", () => {
  describe("formatIsoWeek", () => {
    it("formats year + week to YYYY-Www with zero padding", () => {
      expect(formatIsoWeek({ year: 2026, week: 17 })).toBe("2026-W17");
      expect(formatIsoWeek({ year: 2026, week: 1 })).toBe("2026-W01");
      expect(formatIsoWeek({ year: 2025, week: 52 })).toBe("2025-W52");
    });
  });

  describe("parseIsoWeek", () => {
    it("parses YYYY-Www into year + week", () => {
      expect(parseIsoWeek("2026-W17")).toEqual({ year: 2026, week: 17 });
      expect(parseIsoWeek("2025-W01")).toEqual({ year: 2025, week: 1 });
    });

    it("throws on malformed input", () => {
      expect(() => parseIsoWeek("2026-17")).toThrow();
      expect(() => parseIsoWeek("not-a-week")).toThrow();
      expect(() => parseIsoWeek("2026-W00")).toThrow();
      expect(() => parseIsoWeek("2026-W54")).toThrow();
    });
  });

  describe("addWeeks", () => {
    it("moves forward within a year", () => {
      expect(addWeeks({ year: 2026, week: 10 }, 4)).toEqual({ year: 2026, week: 14 });
    });

    it("moves backward within a year", () => {
      expect(addWeeks({ year: 2026, week: 10 }, -3)).toEqual({ year: 2026, week: 7 });
    });

    it("crosses year boundary forward", () => {
      expect(addWeeks({ year: 2025, week: 51 }, 3)).toEqual({ year: 2026, week: 2 });
    });

    it("crosses year boundary backward", () => {
      expect(addWeeks({ year: 2026, week: 2 }, -3)).toEqual({ year: 2025, week: 51 });
    });

    it("handles 53-week ISO years (2020 had W53)", () => {
      // 2020 is a 53-week year. Going from 2020-W52 +1 should land on 2020-W53.
      expect(addWeeks({ year: 2020, week: 52 }, 1)).toEqual({ year: 2020, week: 53 });
      expect(addWeeks({ year: 2020, week: 53 }, 1)).toEqual({ year: 2021, week: 1 });
    });
  });

  describe("weeksInRange", () => {
    it("yields every week from start to end inclusive", () => {
      const range = weeksInRange({ year: 2026, week: 10 }, { year: 2026, week: 13 });
      expect(range).toEqual([
        { year: 2026, week: 10 },
        { year: 2026, week: 11 },
        { year: 2026, week: 12 },
        { year: 2026, week: 13 },
      ]);
    });

    it("crosses year boundaries", () => {
      const range = weeksInRange({ year: 2025, week: 52 }, { year: 2026, week: 2 });
      expect(range).toEqual([
        { year: 2025, week: 52 },
        { year: 2026, week: 1 },
        { year: 2026, week: 2 },
      ]);
    });

    it("yields a single week when start == end", () => {
      expect(weeksInRange({ year: 2026, week: 17 }, { year: 2026, week: 17 })).toEqual([
        { year: 2026, week: 17 },
      ]);
    });
  });

  describe("currentIsoWeek", () => {
    it("returns a plausible year + week from the system clock", () => {
      const w = currentIsoWeek();
      expect(w.year).toBeGreaterThanOrEqual(2024);
      expect(w.year).toBeLessThanOrEqual(2100);
      expect(w.week).toBeGreaterThanOrEqual(1);
      expect(w.week).toBeLessThanOrEqual(53);
    });
  });
});
