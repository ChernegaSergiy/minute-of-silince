// Mirror of Rust `Settings` struct

export type AudioPreset =
  | "voice_metronome"
  | "metronome_only"
  | "voice_silence_bell"
  | "voice_silence"
  | "voice_metronome_anthem"
  | "metronome_anthem"
  | "bell_silence_bell"
  | "bell_metronome_bell";

export interface Settings {
  ceremonyEnabled: boolean;
  autostartEnabled: boolean;
  weekdaysOnly: boolean;
  preset: AudioPreset;
  volume: number; // 0–100
  pauseOtherPlayers: boolean;
  systemTimeOnly: boolean;
  volumePriority: boolean;
  autoUnmute: boolean;
  ntpServer: string;
  lateStartGraceMinutes: number; // 0–15
}

// Mirror of Rust `StatusSnapshot` struct

export interface StatusSnapshot {
  ceremonyActive: boolean;
  skipTomorrow: boolean;
  lastActivation: string | null;
  lastNtpSync: string | null;
}

// UI helpers

export const PRESET_LABELS: Record<AudioPreset, string> = {
  voice_metronome: "Голос + метроном",
  metronome_only: "Метроном",
  voice_silence_bell: "Голос + тиша + дзвін",
  voice_silence: "Голос + тиша",
  voice_metronome_anthem: "Голос + метроном + гімн",
  metronome_anthem: "Метроном + гімн",
  bell_silence_bell: "Дзвін + тиша + дзвін",
  bell_metronome_bell: "Дзвін + метроном + дзвін",
};

export const DEFAULT_SETTINGS: Settings = {
  ceremonyEnabled: true,
  autostartEnabled: true,
  weekdaysOnly: false,
  preset: "voice_metronome",
  volume: 80,
  pauseOtherPlayers: true,
  systemTimeOnly: false,
  volumePriority: false,
  autoUnmute: false,
  ntpServer: "pool.ntp.org",
  lateStartGraceMinutes: 1,
};
