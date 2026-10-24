/**
 * Native file selection and drag-and-drop for imports.
 *
 * Paths come from the operating system (the system open dialog, or files dropped on the window),
 * never from typing. Only paths are passed on; the Rust side reads, validates and encrypts the file.
 */

import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebview } from "@tauri-apps/api/webview";

export const IMPORT_EXTENSIONS = ["pdf", "png", "jpg", "jpeg", "csv"];

export const baseName = (path: string): string => path.split(/[\\/]/).pop() ?? path;

export const isSupportedImport = (path: string): boolean => {
  const ext = baseName(path).split(".").pop()?.toLowerCase() ?? "";
  return IMPORT_EXTENSIONS.includes(ext);
};

type OpenResult = string | string[] | null;

async function open(options: Record<string, unknown>): Promise<string[]> {
  const result = await invoke<OpenResult>("plugin:dialog|open", { options });
  if (result === null) return [];
  return Array.isArray(result) ? result : [result];
}

/** System open dialog for documents; returns the chosen paths (empty when cancelled). */
export function pickImportFiles(): Promise<string[]> {
  return open({
    multiple: true,
    directory: false,
    title: "Import documents",
    filters: [
      { name: "Invoices, receipts and statements", extensions: IMPORT_EXTENSIONS },
    ],
  });
}

/** System open dialog for a SAAR backup archive. */
export async function pickBackupArchive(): Promise<string | null> {
  const paths = await open({
    multiple: false,
    directory: false,
    title: "Choose a SAAR backup",
    filters: [{ name: "SAAR backup", extensions: ["saar"] }],
  });
  return paths[0] ?? null;
}

/**
 * Calls back while files are dragged over the window and with their paths when dropped.
 * Returns a function that removes the listener.
 */
export async function listenForDroppedFiles(
  onHover: (hovering: boolean) => void,
  onDrop: (paths: string[]) => void,
): Promise<() => void> {
  return getCurrentWebview().onDragDropEvent((event) => {
    const payload = event.payload;
    if (payload.type === "enter" || payload.type === "over") onHover(true);
    else if (payload.type === "leave") onHover(false);
    else if (payload.type === "drop") {
      onHover(false);
      onDrop(payload.paths);
    }
  });
}
