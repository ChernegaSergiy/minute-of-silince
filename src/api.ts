/**
 * Typed wrappers around Tauri `invoke` calls.
 *
 * All functions in this module correspond 1-to-1 with a `#[tauri::command]`
 * in `src-tauri/src/commands.rs`.
 */

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { Settings, StatusSnapshot } from "./types";

// ── Settings ────────────────────────────────────────────────────────────────

export async function getSettings(): Promise<Settings> {
  return invoke<Settings>("get_settings");
}

export async function saveSettings(settings: Settings): Promise<void> {
  return invoke<void>("save_settings", { settings });
}

// ── Status ──────────────────────────────────────────────────────────────────

export async function getStatus(): Promise<StatusSnapshot> {
  return invoke<StatusSnapshot>("get_status");
}

// ── Skip / unskip ───────────────────────────────────────────────────────────

export async function skipNext(): Promise<void> {
  return invoke<void>("skip_next");
}

export async function unskipNext(): Promise<void> {
  return invoke<void>("unskip_next");
}

// ── Manual trigger ──────────────────────────────────────────────────────────

export async function triggerCeremonyNow(): Promise<void> {
  return invoke<void>("trigger_ceremony_now");
}

// ── Event listeners ─────────────────────────────────────────────────────────

export const CEREMONY_START_EVENT = "ceremony:start";
export const CEREMONY_END_EVENT = "ceremony:end";

export function onCeremonyStart(callback: () => void): Promise<UnlistenFn> {
  return listen(CEREMONY_START_EVENT, callback);
}

export function onCeremonyEnd(callback: () => void): Promise<UnlistenFn> {
  return listen(CEREMONY_END_EVENT, callback);
}
