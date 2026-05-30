/**
 * Typed wrappers around Tauri `invoke` calls.
 *
 * All functions in this module correspond 1-to-1 with a `#[tauri::command]`
 * in `src-tauri/src/commands.rs`.
 */

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { LazyStore } from "@tauri-apps/plugin-store";
import { DEFAULT_SETTINGS, type PersonalDate, type Settings, type StatusSnapshot } from "./types";

// Settings

const store = new LazyStore("settings.json");

export async function getSettings(): Promise<Settings> {
  const settings = await store.get<Settings>("settings");
  return settings ?? { ...DEFAULT_SETTINGS };
}

export async function saveSettings(settings: Settings): Promise<void> {
  await store.set("settings", settings);
  await store.save();
}

// Personal Dates

export async function getPersonalDates(): Promise<PersonalDate[]> {
  const dates = await store.get<PersonalDate[]>("personal_dates");
  return dates ?? [];
}

export async function savePersonalDates(dates: PersonalDate[]): Promise<void> {
  await store.set("personal_dates", dates);
  await store.save();
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
