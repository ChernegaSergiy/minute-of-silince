import type { AudioPreset } from "./types";

class AudioEngine {
  private currentAudio: HTMLAudioElement | null = null;
  private stopRequested = false;
  private waitTimeout: number | null = null;
  private waitResolve: (() => void) | null = null;

  private wait(ms: number): Promise<void> {
    return new Promise((resolve) => {
      if (this.stopRequested) {
        resolve();
        return;
      }
      this.waitResolve = resolve;
      this.waitTimeout = window.setTimeout(() => {
        this.waitTimeout = null;
        this.waitResolve = null;
        resolve();
      }, ms);
    });
  }

  private playFile(url: string, volume: number): Promise<void> {
    console.log(`AudioEngine: preparing to play ${url}`);
    return new Promise((resolve) => {
      if (this.stopRequested) {
        console.log("AudioEngine: stop requested before playing");
        resolve();
        return;
      }
      const audio = new Audio(url);
      audio.volume = volume / 100;
      this.currentAudio = audio;

      audio.onplay = () => console.log(`AudioEngine: playing ${url}`);
      
      audio.onended = () => {
        console.log(`AudioEngine: finished ${url}`);
        this.currentAudio = null;
        resolve();
      };

      audio.onerror = (e) => {
        console.error(`AudioEngine: error for ${url}:`, e);
        this.currentAudio = null;
        resolve();
      };

      audio.play().catch((e) => {
        console.error(`AudioEngine: failed to play ${url}:`, e);
        this.currentAudio = null;
        resolve();
      });
    });
  }

  public stop(): void {
    this.stopRequested = true;
    if (this.currentAudio) {
      this.currentAudio.pause();
      this.currentAudio = null;
    }
    if (this.waitTimeout !== null && this.waitResolve) {
      window.clearTimeout(this.waitTimeout);
      this.waitTimeout = null;
      this.waitResolve();
      this.waitResolve = null;
    }
  }

  public async playPreset(preset: AudioPreset, volume: number): Promise<void> {
    this.stopRequested = false;

    try {
      switch (preset) {
        case "voice_metronome":
          await this.playFile("/audio/announcement_with_metronome.ogg", volume);
          break;
        case "metronome_only":
          await this.playFile("/audio/metronome.ogg", volume);
          break;
        case "voice_silence_bell":
          await this.playFile("/audio/announcement.ogg", volume);
          await this.wait(60000);
          await this.playFile("/audio/bell.ogg", volume);
          break;
        case "voice_silence":
          await this.playFile("/audio/announcement.ogg", volume);
          await this.wait(60000);
          break;
        case "voice_metronome_anthem":
          await this.playFile("/audio/announcement_with_metronome.ogg", volume);
          await this.playFile("/audio/anthem.ogg", volume);
          break;
        case "metronome_anthem":
          await this.playFile("/audio/metronome.ogg", volume);
          await this.playFile("/audio/anthem.ogg", volume);
          break;
        case "bell_silence_bell":
          await this.playFile("/audio/bell.ogg", volume);
          await this.wait(60000);
          await this.playFile("/audio/bell.ogg", volume);
          break;
        case "bell_metronome_bell":
          await this.playFile("/audio/bell.ogg", volume);
          await this.playFile("/audio/metronome.ogg", volume);
          await this.playFile("/audio/bell.ogg", volume);
          break;
      }
    } finally {
      this.stopRequested = false;
    }
  }
}

export const audioPlayer = new AudioEngine();
