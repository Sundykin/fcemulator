// NES 2C02 master palette (64 colors, RGB hex). Used by the CHR/map editors to
// render 4-color palette slots. (A common reference variant.)
export const NES_PALETTE: string[] = [
  "#626262","#001FB2","#2404C8","#5200B2","#730076","#800024","#730B00","#522800",
  "#244400","#005700","#005C00","#005324","#003C76","#000000","#000000","#000000",
  "#ABABAB","#0D57FF","#4B30FF","#8A13FF","#BC08D6","#D21269","#C72E00","#9D5400",
  "#607B00","#209800","#00A300","#009942","#007DB4","#000000","#000000","#000000",
  "#FFFFFF","#53AEFF","#9085FF","#D365FF","#FF57FF","#FF5DCF","#FF7757","#FA9E00",
  "#BDC700","#7AE700","#43F611","#26EF7E","#2CD5F6","#4E4E4E","#000000","#000000",
  "#FFFFFF","#B6E1FF","#CED1FF","#E9C3FF","#FFBCFF","#FFBDF4","#FFC6C3","#FFD59A",
  "#E9E681","#CEF481","#B3FB9A","#9EFBC2","#A0EFF7","#BEBEBE","#000000","#000000",
];

// Default 4-slot working palette (indices into NES_PALETTE): bg + 3 shades.
export const DEFAULT_PALETTE: number[] = [0x0f, 0x00, 0x10, 0x30];
