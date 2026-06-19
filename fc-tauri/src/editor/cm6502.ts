// CodeMirror 6 setup for 6502 / ca65 assembly: a small StreamLanguage, syntax
// highlighting matched to the app theme, mnemonic / register / directive
// completion, and .proc/.scope-style folding. (M1 code-editor capability.)
import { EditorState, StateField, StateEffect, RangeSetBuilder, type Extension } from "@codemirror/state";
import {
  EditorView,
  keymap,
  lineNumbers,
  highlightActiveLine,
  highlightActiveLineGutter,
  drawSelection,
  gutter,
  GutterMarker,
} from "@codemirror/view";
import { defaultKeymap, history, historyKeymap, indentWithTab } from "@codemirror/commands";
import {
  StreamLanguage,
  HighlightStyle,
  syntaxHighlighting,
  indentOnInput,
  bracketMatching,
  foldGutter,
  codeFolding,
  foldKeymap,
  foldService,
} from "@codemirror/language";
import {
  autocompletion,
  completionKeymap,
  closeBrackets,
  closeBracketsKeymap,
  type CompletionContext,
  type CompletionResult,
  type Completion,
} from "@codemirror/autocomplete";
import { tags as t } from "@lezer/highlight";

// 6502 official mnemonics.
const MNEMONICS = [
  "adc","and","asl","bcc","bcs","beq","bit","bmi","bne","bpl","brk","bvc","bvs",
  "clc","cld","cli","clv","cmp","cpx","cpy","dec","dex","dey","eor","inc","inx",
  "iny","jmp","jsr","lda","ldx","ldy","lsr","nop","ora","pha","php","pla","plp",
  "rol","ror","rti","rts","sbc","sec","sed","sei","sta","stx","sty","tax","tay",
  "tsx","txa","txs","tya",
];

// Common ca65 directives.
const DIRECTIVES = [
  ".segment",".byte",".word",".dbyt",".addr",".res",".asciiz",".include",".incbin",
  ".proc",".endproc",".scope",".endscope",".macro",".endmacro",".enum",".endenum",
  ".struct",".endstruct",".repeat",".endrep",".if",".else",".elseif",".endif",
  ".ifdef",".ifndef",".import",".export",".importzp",".exportzp",".global",".globalzp",
  ".define",".org",".align",".setcpu",".feature",".zeropage",".bss",".data",".rodata",
  ".code",".out",".warning",".error",".a8",".a16",".i8",".i16",".pushseg",".popseg",
];

// NES PPU/APU register constants (name → address).
const REGISTERS: [string, number][] = [
  ["PPUCTRL", 0x2000],["PPUMASK", 0x2001],["PPUSTATUS", 0x2002],["OAMADDR", 0x2003],
  ["OAMDATA", 0x2004],["PPUSCROLL", 0x2005],["PPUADDR", 0x2006],["PPUDATA", 0x2007],
  ["OAMDMA", 0x4014],["SQ1_VOL", 0x4000],["SQ1_SWEEP", 0x4001],["SQ1_LO", 0x4002],
  ["SQ1_HI", 0x4003],["SQ2_VOL", 0x4004],["SQ2_SWEEP", 0x4005],["SQ2_LO", 0x4006],
  ["SQ2_HI", 0x4007],["TRI_LINEAR", 0x4008],["TRI_LO", 0x400a],["TRI_HI", 0x400b],
  ["NOISE_VOL", 0x400c],["NOISE_LO", 0x400e],["NOISE_HI", 0x400f],["DMC_FREQ", 0x4010],
  ["DMC_RAW", 0x4011],["DMC_START", 0x4012],["DMC_LEN", 0x4013],["APU_STATUS", 0x4015],
  ["JOY1", 0x4016],["JOY2", 0x4017],
];

const MNEMONIC_SET = new Set(MNEMONICS);
const DIRECTIVE_SET = new Set(DIRECTIVES);

// ---------------------------------------------------------------- language

const ca65Mode = StreamLanguage.define<{}>({
  name: "ca65",
  startState: () => ({}),
  token(stream) {
    if (stream.eatSpace()) return null;
    const ch = stream.peek();
    // comments
    if (ch === ";") {
      stream.skipToEnd();
      return "comment";
    }
    // strings / char literals
    if (ch === '"' || ch === "'") {
      const quote = stream.next();
      let escaped = false;
      let c: string | void;
      while ((c = stream.next()) != null) {
        if (c === quote && !escaped) break;
        escaped = !escaped && c === "\\";
      }
      return "string";
    }
    // immediate / operators
    if (ch === "#") {
      stream.next();
      return "operator";
    }
    // numbers: $hex, %bin, decimal
    if (ch === "$") {
      stream.next();
      stream.eatWhile(/[0-9a-fA-F]/);
      return "number";
    }
    if (ch === "%") {
      stream.next();
      stream.eatWhile(/[01]/);
      return "number";
    }
    if (/[0-9]/.test(ch as string)) {
      stream.eatWhile(/[0-9]/);
      return "number";
    }
    // ca65 directives (.segment, .byte, ...)
    if (ch === ".") {
      stream.next();
      stream.eatWhile(/[a-zA-Z0-9_]/);
      return "keyword";
    }
    // identifiers: mnemonic / label / symbol
    if (/[a-zA-Z_@]/.test(ch as string)) {
      stream.eatWhile(/[a-zA-Z0-9_@]/);
      const word = stream.current();
      if (stream.peek() === ":") return "labelName";
      if (MNEMONIC_SET.has(word.toLowerCase())) return "keyword";
      return "variableName";
    }
    stream.next();
    return null;
  },
  languageData: {
    commentTokens: { line: ";" },
    autocomplete: ca65Completion,
  },
});

// --------------------------------------------------------------- completion

const COMPLETIONS: Completion[] = [
  ...MNEMONICS.map((m): Completion => ({ label: m.toUpperCase(), type: "keyword", detail: "指令" })),
  ...DIRECTIVES.map((d): Completion => ({ label: d, type: "keyword", detail: "ca65 伪指令" })),
  ...REGISTERS.map(([name, addr]): Completion => ({
    label: name,
    type: "constant",
    detail: "$" + addr.toString(16).toUpperCase().padStart(4, "0"),
  })),
];

export function ca65Completion(context: CompletionContext): CompletionResult | null {
  const word = context.matchBefore(/[.A-Za-z_@][\w.@]*/);
  if (!word || (word.from === word.to && !context.explicit)) return null;
  return { from: word.from, options: COMPLETIONS, validFor: /^[.\w@]*$/ };
}

// ----------------------------------------------------------------- folding

// Fold .proc/.scope/.macro/.enum/.struct/.repeat/.if regions (depth-aware).
const FOLD_PAIRS: Record<string, string> = {
  proc: "endproc",
  scope: "endscope",
  macro: "endmacro",
  enum: "endenum",
  struct: "endstruct",
  repeat: "endrep",
  if: "endif",
};

const ca65Fold = foldService.of((state, lineStart, lineEnd) => {
  const line = state.doc.lineAt(lineStart);
  const m = /^\s*\.(\w+)/.exec(line.text);
  if (!m) return null;
  const opener = m[1].toLowerCase();
  const closer = FOLD_PAIRS[opener];
  if (!closer) return null;
  let depth = 1;
  for (let n = line.number + 1; n <= state.doc.lines; n++) {
    const ln = state.doc.line(n);
    const dm = /^\s*\.(\w+)/.exec(ln.text);
    if (dm) {
      const d = dm[1].toLowerCase();
      if (d === opener) depth++;
      else if (d === closer) {
        depth--;
        if (depth === 0) return { from: lineEnd, to: ln.from - 1 };
      }
    }
  }
  return null;
});

// ------------------------------------------------------------------- theme

const editorTheme = EditorView.theme(
  {
    "&": { backgroundColor: "transparent", color: "var(--text)", height: "100%" },
    ".cm-content": { fontFamily: "var(--font-mono, monospace)", fontSize: "13px", caretColor: "var(--accent)" },
    ".cm-gutters": { backgroundColor: "transparent", color: "var(--text-mute)", border: "none" },
    ".cm-activeLine": { backgroundColor: "rgba(124,92,255,0.06)" },
    ".cm-activeLineGutter": { backgroundColor: "rgba(124,92,255,0.10)", color: "var(--text-dim)" },
    "&.cm-focused .cm-selectionBackground, .cm-selectionBackground": { backgroundColor: "rgba(124,92,255,0.22)" },
    ".cm-cursor": { borderLeftColor: "var(--accent)" },
    ".cm-tooltip": { background: "var(--panel)", border: "1px solid var(--border)", color: "var(--text)" },
    ".cm-tooltip-autocomplete ul li[aria-selected]": { background: "var(--accent-soft)", color: "var(--accent)" },
    ".cm-bp-gutter": { width: "14px", cursor: "pointer" },
    ".cm-bp-dot": { color: "var(--danger)", fontSize: "11px", lineHeight: "1.4", textAlign: "center" },
    ".cm-bp-spacer": { visibility: "hidden", fontSize: "11px", lineHeight: "1.4" },
    ".cm-bp-gutter .cm-gutterElement:hover": { background: "rgba(244,63,94,0.15)" },
    ".cm-halt-line": { background: "rgba(251,191,36,0.18)" },
  },
  { dark: true }
);

const highlight = HighlightStyle.define([
  { tag: t.comment, color: "var(--text-mute)", fontStyle: "italic" },
  { tag: t.keyword, color: "#a368ff" },
  { tag: t.string, color: "#4ade80" },
  { tag: t.number, color: "#38bdf8" },
  { tag: t.operator, color: "#fbbf24" },
  { tag: t.labelName, color: "#fbbf24" },
  { tag: t.variableName, color: "var(--text)" },
]);

// -------------------------------------------------------------- breakpoints

const toggleBp = StateEffect.define<{ line: number; on: boolean }>();
const bpState = StateField.define<Set<number>>({
  create: () => new Set(),
  update(set, tr) {
    let next = set;
    for (const e of tr.effects) {
      if (e.is(toggleBp)) {
        next = new Set(next);
        if (e.value.on) next.add(e.value.line);
        else next.delete(e.value.line);
      }
    }
    return next;
  },
});

class BpMarker extends GutterMarker {
  toDOM() {
    const d = document.createElement("div");
    d.className = "cm-bp-dot";
    d.textContent = "●";
    return d;
  }
}
const bpMarker = new BpMarker();

// Invisible marker that only reserves gutter width (the visible dot is bpMarker).
class SpacerMarker extends GutterMarker {
  toDOM() {
    const d = document.createElement("div");
    d.className = "cm-bp-spacer";
    d.textContent = "●";
    return d;
  }
}
const spacerMarker = new SpacerMarker();

/**
 * A breakpoint gutter: starts with `initial` breakpoint lines (1-based), toggles
 * on click, and reports each change via `onToggle`. (M1 source-debug-link 6.3.)
 */
export function breakpointGutter(
  initial: number[],
  onToggle: (line: number, on: boolean) => void
): Extension {
  return [
    bpState.init(() => new Set(initial)),
    gutter({
      class: "cm-bp-gutter",
      markers: (view) => {
        const set = view.state.field(bpState);
        const b = new RangeSetBuilder<GutterMarker>();
        [...set]
          .sort((a, c) => a - c)
          .forEach((ln) => {
            if (ln >= 1 && ln <= view.state.doc.lines) {
              b.add(view.state.doc.line(ln).from, view.state.doc.line(ln).from, bpMarker);
            }
          });
        return b.finish();
      },
      initialSpacer: () => spacerMarker,
      domEventHandlers: {
        mousedown(view, line) {
          const ln = view.state.doc.lineAt(line.from).number;
          const on = !view.state.field(bpState).has(ln);
          view.dispatch({ effects: toggleBp.of({ line: ln, on }) });
          onToggle(ln, on);
          return true;
        },
      },
    }),
  ];
}

// --------------------------------------------------------------- assembly

/** Build the full extension list for an editor. `onChange` fires on edits. */
export function ca65Extensions(onChange: (doc: string) => void): Extension {
  return [
    lineNumbers(),
    highlightActiveLineGutter(),
    highlightActiveLine(),
    drawSelection(),
    history(),
    indentOnInput(),
    bracketMatching(),
    closeBrackets(),
    codeFolding(),
    foldGutter(),
    ca65Fold,
    ca65Mode,
    syntaxHighlighting(highlight),
    editorTheme,
    autocompletion({ override: [ca65Completion] }),
    keymap.of([
      indentWithTab,
      ...closeBracketsKeymap,
      ...defaultKeymap,
      ...historyKeymap,
      ...foldKeymap,
      ...completionKeymap,
    ]),
    EditorView.updateListener.of((v) => {
      if (v.docChanged) onChange(v.state.doc.toString());
    }),
  ];
}

export function makeEditorState(doc: string, onChange: (doc: string) => void): EditorState {
  return EditorState.create({ doc, extensions: [ca65Extensions(onChange)] });
}
