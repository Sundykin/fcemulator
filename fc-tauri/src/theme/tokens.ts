// Design tokens — deep-navy base with violet (#7C5CFF) accent, matching the
// reference design files in ui设计/. Mirrored as CSS variables in
// styles/global.css (keep the two in sync).

export const color = {
  bg: "#0a0e1a", // app background (deep navy)
  bar: "#0e1322", // title bar / toolbar / footer
  panel: "#141a2b", // cards, side panels
  surface: "#1a2135", // raised surface (inputs, buttons)
  surfaceHover: "#212a42",
  border: "#242c42",
  borderStrong: "#34406a",
  text: "#e6e9f2",
  textDim: "#99a1b6",
  textMute: "#5e6781",
  accent: "#7c5cff", // violet primary
  accentHover: "#9170ff",
  accentSoft: "rgba(124,92,255,0.14)",
  accentGlow: "rgba(124,92,255,0.5)",
  accentGradFrom: "#7c5cff",
  accentGradTo: "#a368ff",
  green: "#4ade80", // FPS / running indicator
  cyan: "#38bdf8",
  warning: "#fbbf24",
  danger: "#f43f5e",
} as const;

export const radius = { sm: "7px", md: "10px", lg: "14px", pill: "999px" } as const;
export const space = { xs: "4px", sm: "8px", md: "12px", lg: "16px", xl: "24px", xxl: "32px" } as const;

export const font = {
  ui: `-apple-system, "PingFang SC", "Segoe UI", system-ui, sans-serif`,
  mono: `"SF Mono", "JetBrains Mono", ui-monospace, Menlo, monospace`,
} as const;

export const shadow = {
  card: "0 4px 18px rgba(0,0,0,0.4)",
  pop: "0 10px 34px rgba(0,0,0,0.5)",
  glow: "0 0 0 1px rgba(124,92,255,0.5), 0 8px 28px rgba(124,92,255,0.4)",
} as const;
