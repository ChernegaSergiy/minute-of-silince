/**
 * Typed wrappers around Tauri `invoke` calls.
 *
 * All functions in this module correspond 1-to-1 with a `#[tauri::command]`
 * in `src-tauri/src/commands.rs`.
 */

import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { Settings, StatusSnapshot } from "./types";

declare global {
  interface Window {
    __TAURI__?: {
      core?: {
        invoke: <T = unknown>(cmd: string, args?: Record<string, unknown>) => Promise<T>;
      };
    };
  }
}

function getInvoke() {
  if (typeof window !== 'undefined' && window.__TAURI__?.core?.invoke) {
    return window.__TAURI__.core.invoke;
  }
  throw new Error("Tauri invoke not available");
}

async function invoke<T = unknown>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  return getInvoke()<T>(cmd, args);
}

// Settings

export async function getSettings(): Promise<Settings> {
  return invoke<Settings>("get_settings");
}

export async function saveSettings(settings: Settings): Promise<void> {
  return invoke<void>("save_settings", { settings });
}

// Status

export async function getStatus(): Promise<StatusSnapshot> {
  return invoke<StatusSnapshot>("get_status");
}

export async function syncNtpNow(): Promise<StatusSnapshot> {
  return invoke<StatusSnapshot>("sync_ntp_now");
}

// Skip / unskip

export async function skipNext(): Promise<void> {
  return invoke<void>("skip_next");
}

export async function unskipNext(): Promise<void> {
  return invoke<void>("unskip_next");
}

// Manual trigger

export async function triggerCeremonyNow(): Promise<void> {
  return invoke<void>("trigger_ceremony_now");
}

export async function finishCeremonyNow(): Promise<void> {
  return invoke<void>("finish_ceremony_now");
}

export async function getLogContents(): Promise<string> {
  return invoke<string>("get_log_contents");
}

// Event listeners

export const CEREMONY_START_EVENT = "ceremony-start";
export const CEREMONY_END_EVENT = "ceremony-end";

export type CeremonyStartPayload = { duration_ms?: number };

export function onCeremonyStart(callback: (payload: CeremonyStartPayload) => void): Promise<UnlistenFn> {
  return listen(CEREMONY_START_EVENT, (e) => callback((e.payload as unknown) as CeremonyStartPayload));
}

export function onCeremonyEnd(callback: () => void): Promise<UnlistenFn> {
  return listen(CEREMONY_END_EVENT, callback);
}

export async function bringWindowToFront(): Promise<void> {
  const win = getCurrentWindow();
  const isMin = await win.isMinimized();
  if (isMin) {
    await win.unminimize();
  }
  const isVisible = await win.isVisible();
  if (!isVisible) {
    await win.show();
  }
  await win.setFocus();
}
