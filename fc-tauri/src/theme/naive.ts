// Naive UI global theme overrides, derived from the design tokens.
import type { GlobalThemeOverrides } from "naive-ui";
import { color, radius, font } from "./tokens";

export const naiveOverrides: GlobalThemeOverrides = {
  common: {
    primaryColor: color.accent,
    primaryColorHover: color.accentHover,
    primaryColorPressed: color.accentHover,
    primaryColorSuppl: color.accentHover,
    bodyColor: color.bg,
    cardColor: color.panel,
    modalColor: color.panel,
    popoverColor: color.surface,
    tableColor: color.panel,
    inputColor: color.surface,
    borderColor: color.border,
    dividerColor: color.border,
    textColorBase: color.text,
    textColor1: color.text,
    textColor2: color.textDim,
    textColor3: color.textMute,
    borderRadius: radius.md,
    borderRadiusSmall: radius.sm,
    fontFamily: font.ui,
    fontFamilyMono: font.mono,
  },
  Button: { borderRadiusMedium: radius.md, borderRadiusSmall: radius.sm },
  Card: { borderRadius: radius.lg, paddingMedium: "16px" },
  Tag: { borderRadius: radius.pill },
  Select: { peers: { InternalSelection: { borderRadius: radius.sm } } },
};
