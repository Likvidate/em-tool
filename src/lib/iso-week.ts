export interface IsoWeek {
  year: number;
  week: number;
}

/**
 * Returns the Monday date (UTC) of the ISO 8601 week containing `date`.
 */
function mondayOfIsoWeek(date: Date): Date {
  const d = new Date(Date.UTC(date.getUTCFullYear(), date.getUTCMonth(), date.getUTCDate()));
  // ISO: Monday=1, Sunday=7
  const dow = d.getUTCDay() || 7;
  d.setUTCDate(d.getUTCDate() - (dow - 1));
  return d;
}

/**
 * Returns the ISO 8601 week number and week-year for the given date.
 * The week-year can differ from the calendar year near Jan 1 / Dec 31.
 */
function isoWeekOf(date: Date): IsoWeek {
  const monday = mondayOfIsoWeek(date);
  const thursday = new Date(monday);
  thursday.setUTCDate(monday.getUTCDate() + 3); // Thursday of this ISO week
  const year = thursday.getUTCFullYear();
  const firstThursday = new Date(Date.UTC(year, 0, 4));
  const firstMonday = mondayOfIsoWeek(firstThursday);
  const diffMs = monday.getTime() - firstMonday.getTime();
  const week = 1 + Math.round(diffMs / (7 * 24 * 60 * 60 * 1000));
  return { year, week };
}

/**
 * Returns true if `year` has 53 ISO weeks.
 * An ISO year has 53 weeks iff Jan 1 falls on Thursday, or Jan 1 falls on
 * Wednesday in a leap year.
 */
function weeksInIsoYear(year: number): 52 | 53 {
  const jan1 = new Date(Date.UTC(year, 0, 1)).getUTCDay(); // 0=Sun..6=Sat
  const isLeap = (year % 4 === 0 && year % 100 !== 0) || year % 400 === 0;
  if (jan1 === 4 || (jan1 === 3 && isLeap)) return 53;
  return 52;
}

export function currentIsoWeek(): IsoWeek {
  return isoWeekOf(new Date());
}

export function formatIsoWeek({ year, week }: IsoWeek): string {
  const ww = String(week).padStart(2, "0");
  return `${year}-W${ww}`;
}

export function parseIsoWeek(s: string): IsoWeek {
  const match = /^(\d{4})-W(\d{2})$/.exec(s);
  if (!match) throw new Error(`Invalid ISO week string: ${s}`);
  const year = Number(match[1]);
  const week = Number(match[2]);
  if (week < 1 || week > weeksInIsoYear(year)) {
    throw new Error(`Invalid ISO week number: ${s}`);
  }
  return { year, week };
}

export function addWeeks(w: IsoWeek, delta: number): IsoWeek {
  let { year, week } = w;
  week += delta;
  while (week < 1) {
    year -= 1;
    week += weeksInIsoYear(year);
  }
  while (week > weeksInIsoYear(year)) {
    week -= weeksInIsoYear(year);
    year += 1;
  }
  return { year, week };
}

function compareIsoWeek(a: IsoWeek, b: IsoWeek): number {
  if (a.year !== b.year) return a.year - b.year;
  return a.week - b.week;
}

export function weeksInRange(start: IsoWeek, end: IsoWeek): IsoWeek[] {
  if (compareIsoWeek(start, end) > 0) {
    throw new Error("start must be <= end");
  }
  const out: IsoWeek[] = [];
  let cur = start;
  while (compareIsoWeek(cur, end) <= 0) {
    out.push(cur);
    cur = addWeeks(cur, 1);
  }
  return out;
}
