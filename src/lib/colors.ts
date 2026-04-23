export const COLOR_ORDER = ["red", "yellow", "grey", "green", "blue"] as const;
export type Color = typeof COLOR_ORDER[number];

export interface ColorDef {
  key: Color;
  label: string;
  hex: string;
  description: string;
}

export const COLORS: Record<Color, ColorDef> = {
  red:    { key: "red",    label: "Red",    hex: "#ef4444", description: "Serious issue, flag for next 1:1" },
  yellow: { key: "yellow", label: "Yellow", hex: "#facc15", description: "Concern, keep an eye on" },
  grey:   { key: "grey",   label: "Grey",   hex: "#6b7280", description: "No meaningful signal this week" },
  green:  { key: "green",  label: "Green",  hex: "#4ade80", description: "Good week, delivered well" },
  blue:   { key: "blue",   label: "Blue",   hex: "#3b82f6", description: "Growth milestone / big win" },
};

export function colorHex(c: Color): string {
  return COLORS[c].hex;
}
