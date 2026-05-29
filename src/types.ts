// Mirror of Rust `AnnouncementVoice` enum

export type AnnouncementVoice = "bohdan_hdal" | "sonia_sotnyk" | "dania_khomutovskyi" | "radio_bg" | "air_alert";

// Mirror of Rust `AnthemVoice` enum

export type AnthemVoice = "default" | "mykhailo_khoma" | "oleksandr_ponomarov";

// Mirror of Rust `Settings` struct

export type AudioPreset =
  | "voice_metronome"
  | "metronome_only"
  | "voice_silence_bell"
  | "voice_silence"
  | "voice_metronome_anthem"
  | "voice_metronome_ending"
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
  resumeAfterCeremony: boolean;
  showVisualOverlay: boolean;
  showFlagAnimation: boolean;
  systemTimeOnly: boolean;
  volumePriority: boolean;
  autoUnmute: boolean;
  ntpServer: string;
  lateStartGraceMinutes: number; // 0–5
  /** Enable reminder notifications. */
  reminderEnabled: boolean;
  /** Minutes before 09:00 to show reminder. 0 = immediately. */
  reminderMinutesBefore: number; // 0–10
  /** Selected announcement voice. */
  announcementVoice: AnnouncementVoice;
  /** Selected anthem voice. */
  anthemVoice: AnthemVoice;
  /** Whether to follow the OS theme */
  useSystemTheme?: boolean;
  /** Manual UI theme when not using system theme: 'light' | 'dark' */
  uiTheme?: "light" | "dark";
  /** User-defined personal dates (month/day) */
  personalDates?: PersonalDate[];
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
  resumeAfterCeremony: false,
  showVisualOverlay: true,
  showFlagAnimation: false,
  systemTimeOnly: false,
  volumePriority: false,
  autoUnmute: false,
  ntpServer: "pool.ntp.org",
  lateStartGraceMinutes: 1,
  reminderEnabled: false,
  reminderMinutesBefore: 5,
  announcementVoice: "bohdan_hdal",
  anthemVoice: "default",
  useSystemTheme: true,
  uiTheme: "light",
  personalDates: [],
};

export interface PersonalDate {
  month: number; // 1-12
  day: number; // 1-31
  label: string;
  year: number;
}
