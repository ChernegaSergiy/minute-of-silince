// Mirror of Rust `Settings` struct

export type AudioPreset =
  | "voice_metronome"
  | "metronome_only"
  | "voice_silence_bell"
  | "voice_silence"
  | "voice_metronome_anthem"
  | "metronome_anthem"
  | "bell_silence_bell"
  | "bell_metronome_bell"
  | "silence";

export interface Settings {
  ceremonyEnabled: boolean;
  autostartEnabled: boolean;
  weekdaysOnly: boolean;
  preset: AudioPreset;
  volume: number; // 0–100
  pauseOtherPlayers: boolean;
  showVisualOverlay: boolean;
  systemTimeOnly: boolean;
  volumePriority: boolean;
  autoUnmute: boolean;
  ntpServer: string;
  lateStartGraceMinutes: number; // 0–15
  /** Enable reminder notifications. */
  reminderEnabled: boolean;
  /** Minutes before 09:00 to show reminder. 0 = immediately. */
  reminderMinutesBefore: number; // 0–10
}

// Mirror of Rust `StatusSnapshot` struct

export interface StatusSnapshot {
  ceremonyActive: boolean;
  skipTomorrow: boolean;
  lastActivation: string | null;
  lastNtpSync: string | null;
}

// UI helpers

export const DEFAULT_SETTINGS: Settings = {
  ceremonyEnabled: true,
  autostartEnabled: true,
  weekdaysOnly: false,
  preset: "voice_metronome",
  volume: 80,
  pauseOtherPlayers: true,
  showVisualOverlay: true,
  systemTimeOnly: false,
  volumePriority: false,
  autoUnmute: false,
  ntpServer: "pool.ntp.org",
  lateStartGraceMinutes: 1,
  reminderEnabled: false,
  reminderMinutesBefore: 5,
};
