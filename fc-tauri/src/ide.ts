// Thin wrappers over the M1 project-model + build-pipeline backend commands
// (src-tauri/src/project.rs, build_pipeline.rs). Used by the IDE store.
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

export interface InesHeader {
  mapper: number;
  prg_banks: number;
  chr_banks: number;
  mirroring: string;
  battery: boolean;
}

export interface ProjectManifest {
  name: string;
  sources: string[];
  chr: string[];
  music: string[];
  maps: string[];
  linker_cfg: string | null;
  output: string;
  ines: InesHeader;
}

export interface FileNode {
  name: string;
  path: string; // relative to project root ("" = root)
  is_dir: boolean;
  children: FileNode[];
}

export interface Diagnostic {
  file: string | null;
  line: number | null;
  severity: "error" | "warning";
  message: string;
}

export interface BuildStep {
  tool: string;
  args: string[];
  exit_code: number | null;
  stdout: string;
  stderr: string;
}

export interface LineAddr {
  file: string; // relative to project root
  line: number;
  addr: number; // CPU address
}

export interface BuildResult {
  success: boolean;
  output: string | null;
  log: string;
  diagnostics: Diagnostic[];
  steps: BuildStep[];
  source_map: LineAddr[];
}

export type TemplateId = "blank" | "horizontal" | "demo";

// ---- project ----
export const projectNew = (dir: string, name: string, template: TemplateId) =>
  invoke<ProjectManifest>("project_new", { dir, name, template });
export const projectOpen = (dir: string) => invoke<ProjectManifest>("project_open", { dir });
export const projectGet = () => invoke<ProjectManifest>("project_get");
export const projectSave = (manifest: ProjectManifest) => invoke("project_save", { manifest });
export const projectFileTree = () => invoke<FileNode>("project_file_tree");
export const projectCreateFile = (relPath: string, isDir: boolean) =>
  invoke("project_create_file", { relPath, isDir });
export const projectRenameFile = (from: string, to: string) =>
  invoke("project_rename_file", { from, to });
export const projectDeleteFile = (relPath: string) => invoke("project_delete_file", { relPath });

// project-scoped file read/write for the editor (relative paths, sandboxed in backend)
export const projectReadFile = (relPath: string) => invoke<string>("project_read_file", { relPath });
export const projectWriteFile = (relPath: string, content: string) =>
  invoke("project_write_file", { relPath, content });

// ---- CHR ----
export interface ChrSheet {
  tiles: number;
  pixels: number[]; // tiles × 64, each 0–3
}
export const chrRead = (relPath: string) => invoke<ChrSheet>("chr_read", { relPath });
export const chrWrite = (relPath: string, pixels: number[]) =>
  invoke("chr_write", { relPath, pixels });
export const chrExportInc = (relPath: string, label: string, pixels: number[]) =>
  invoke("chr_export_inc", { relPath, label, pixels });

// ---- map ----
export interface MapData {
  w: number;
  h: number;
  tiles: number[]; // w*h
  attrs: number[]; // ceil(w/2)*ceil(h/2), each 0–3
  collision: number[]; // w*h, 0/1
}
export const mapRead = (relPath: string) => invoke<MapData>("map_read", { relPath });
export const mapWrite = (relPath: string, map: MapData) => invoke("map_write", { relPath, map });

// ---- converters ----
export const convertPngToChr = (srcPath: string, outRel: string) =>
  invoke<number>("convert_png_to_chr", { srcPath, outRel });
export const convertTiledToMap = (srcPath: string, outRel: string) =>
  invoke("convert_tiled_to_map", { srcPath, outRel });

export async function pickFile(name: string, extensions: string[]): Promise<string | null> {
  const p = await open({ multiple: false, filters: [{ name, extensions }] });
  return typeof p === "string" ? p : null;
}

// ---- tracker ----
export interface Instrument { name: string; volume: number[]; arpeggio: number[]; duty: number }
export interface Cell { note: number; instrument: number; volume: number; fx?: number; param?: number }
export interface Pattern { rows: Cell[][] } // rows × 5 channels
export interface Song {
  name: string;
  frames_per_row: number;
  rows_per_pattern: number;
  instruments: Instrument[];
  patterns: Pattern[];
  order: number[];
}
export const NOTE_OFF = 255;
export const trackerSave = (relPath: string, song: Song) => invoke("tracker_save", { relPath, song });
export const trackerLoad = (relPath: string) => invoke<Song>("tracker_load", { relPath });
export const trackerRender = (song: Song, sampleRate: number) =>
  invoke<ArrayBuffer>("tracker_render", { song, sampleRate });
export const trackerExport = (outRel: string, song: Song) =>
  invoke("tracker_export", { outRel, song });
export const trackerImportFtm = (srcPath: string) => invoke<Song>("tracker_import_ftm", { srcPath });

// ---- FamiStudio ----
export const famistudioImport = (sPath: string, dmcPath: string | null) =>
  invoke<string>("famistudio_import", { sPath, dmcPath });

// ---- watch ----
export const watchStart = () => invoke("watch_start");
export const watchStop = () => invoke("watch_stop");

// ---- build ----
export const buildRun = () => invoke<BuildResult>("build_run");
export const buildCancel = () => invoke("build_cancel");

// ---- dialogs ----
export async function pickProjectDir(): Promise<string | null> {
  const d = await open({ directory: true, multiple: false });
  return typeof d === "string" ? d : null;
}
