// ── Mirror of Rust `Settings` struct ────────────────────────────────────────

export type AudioPreset =
  | "voice_silence_bell"
  | "voice_anthem"
  | "voice_metronome"
  | "voice_metronome_anthem"
  | "metronome_only";

export interface Settings {
  autostartEnabled: boolean;
  preset: AudioPreset;
  volume: number; // 0–100
  pauseOtherPlayers: boolean;
  showVisualOverlay: boolean;
  ntpSyncEnabled: boolean;
  ntpServer: string;
  lateStartGraceMinutes: number; // 0–15
}

// ── Mirror of Rust `StatusSnapshot` struct ──────────────────────────────────

export interface StatusSnapshot {
  ceremonyActive: boolean;
  skipTomorrow: boolean;
  lastActivation: string | null;
  lastNtpSync: string | null;
}

// ── UI helpers ───────────────────────────────────────────────────────────────

export const PRESET_LABELS: Record<AudioPreset, string> = {
  voice_silence_bell: "Голос + тиша + дзвін",
  voice_anthem: "Голос + гімн",
  voice_metronome: "Голос + метроном",
  voice_metronome_anthem: "Голос + метроном + гімн",
  metronome_only: "Тільки метроном",
};

export const DEFAULT_SETTINGS: Settings = {
  autostartEnabled: true,
  preset: "voice_silence_bell",
  volume: 80,
  pauseOtherPlayers: true,
  showVisualOverlay: true,
  ntpSyncEnabled: true,
  ntpServer: "pool.ntp.org",
  lateStartGraceMinutes: 5,
};
