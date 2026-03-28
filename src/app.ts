/**
 * Root application controller.
 *
 * Responsibilities:
 *  - Bootstrap the UI from the DOM template in `index.html`.
 *  - Load initial settings and status from the backend.
 *  - Wire up all user-interaction event handlers.
 *  - Subscribe to ceremony events and update the overlay accordingly.
 */

import {
  getSettings,
  getStatus,
  saveSettings,
  skipNext,
  unskipNext,
  triggerCeremonyNow,
  finishCeremonyNow,
  onCeremonyStart,
  onCeremonyEnd,
} from "./api";

import { audioPlayer } from "./audio";
import type { Settings, StatusSnapshot } from "./types";
import { PRESET_LABELS } from "./types";

export class App {
  private root: HTMLElement;
  private settings!: Settings;
  private status!: StatusSnapshot;

  constructor(root: HTMLElement) {
    this.root = root;
  }

  async mount(): Promise<void> {
    try {
      [this.settings, this.status] = await Promise.all([
        getSettings(),
        getStatus(),
      ]);
    } catch (err) {
      console.error("Failed to load initial data from backend:", err);
      return;
    }

    this.render();
    this.bindEvents();
    await this.subscribeToBackendEvents();
  }

  // ── Rendering ─────────────────────────────────────────────────────────────

  private render(): void {
    this.root.innerHTML = `
      <div class="window">
        <header class="window__header">
          <span class="window__title">ХВИЛИНА МОВЧАННЯ</span>
          <span class="window__version">v0.1.0</span>
        </header>

        <main class="window__body">
          <!-- Status badge -->
          <div class="status-badge ${this.status.ceremonyActive ? "status-badge--active" : ""}"
               id="statusBadge">
            ${this.status.ceremonyActive ? "● АКТИВНА ЦЕРЕМОНІЯ" : "○ ОЧІКУВАННЯ"}
          </div>

          <!-- Autostart toggle -->
          <label class="control-row">
            <span class="control-row__label">Автозапуск о 09:00</span>
            <input type="checkbox" id="autostartToggle" class="toggle"
                   ${this.settings.autostartEnabled ? "checked" : ""} />
          </label>

          <!-- Weekdays only toggle -->
          <label class="control-row">
            <span class="control-row__label">Лише в будні</span>
            <input type="checkbox" id="weekdaysToggle" class="toggle"
                   ${this.settings.weekdaysOnly ? "checked" : ""} />
          </label>

          <!-- Skip tomorrow toggle -->
          <label class="control-row">
            <span class="control-row__label">Пропустити завтра</span>
            <input type="checkbox" id="skipToggle" class="toggle"
                   ${this.status.skipTomorrow ? "checked" : ""} />
          </label>

          <hr class="divider" />

          <!-- Audio preset -->
          <div class="control-row">
            <span class="control-row__label">Режим супроводу</span>
            <select id="presetSelect" class="select">
              ${this.renderPresetOptions()}
            </select>
          </div>

          <!-- Volume -->
          <div class="control-row control-row--column">
            <div class="control-row__header">
              <span class="control-row__label">Гучність</span>
              <span class="control-row__value" id="volumeValue">${this.settings.volume}%</span>
            </div>
            <input type="range" id="volumeRange" class="slider"
                   min="0" max="100" value="${this.settings.volume}" />
          </div>

          <hr class="divider" />

          <!-- Pause other players -->
          <label class="control-row">
            <span class="control-row__label">Пауза інших плеєрів</span>
            <input type="checkbox" id="pauseToggle" class="toggle"
                   ${this.settings.pauseOtherPlayers ? "checked" : ""} />
          </label>

          <!-- Visual overlay -->
          <hr class="divider" />

          <!-- Meta / debug info -->
          <div class="meta">
            <span>Остання церемонія: ${this.status.lastActivation ?? "—"}</span>
            <span>Синхронізація NTP: ${this.status.lastNtpSync ?? "—"}</span>
          </div>
        </main>

        <footer class="window__footer">
          <button class="btn btn--ghost" id="testBtn">Тест</button>
          <button class="btn btn--primary" id="saveBtn">Зберегти</button>
        </footer>
      </div>
    `;
  }

  private renderPresetOptions(): string {
    return (Object.keys(PRESET_LABELS) as Array<keyof typeof PRESET_LABELS>)
      .map(
        (key) =>
          `<option value="${key}" ${
            this.settings.preset === key ? "selected" : ""
          }>${PRESET_LABELS[key]}</option>`
      )
      .join("");
  }

  // ── Event wiring ──────────────────────────────────────────────────────────

  private bindEvents(): void {
    this.q<HTMLInputElement>("#autostartToggle").addEventListener(
      "change",
      (e) => {
        this.settings = {
          ...this.settings,
          autostartEnabled: (e.target as HTMLInputElement).checked,
        };
      }
    );

    this.q<HTMLInputElement>("#weekdaysToggle").addEventListener(
      "change",
      (e) => {
        this.settings = {
          ...this.settings,
          weekdaysOnly: (e.target as HTMLInputElement).checked,
        };
      }
    );

    this.q<HTMLInputElement>("#skipToggle").addEventListener("change", (e) => {
      (e.target as HTMLInputElement).checked ? skipNext() : unskipNext();
    });

    this.q<HTMLSelectElement>("#presetSelect").addEventListener(
      "change",
      (e) => {
        this.settings = {
          ...this.settings,
          preset: (e.target as HTMLSelectElement).value as Settings["preset"],
        };
      }
    );

    const volumeRange = this.q<HTMLInputElement>("#volumeRange");
    const volumeValue = this.q<HTMLElement>("#volumeValue");
    volumeRange.addEventListener("input", () => {
      const v = parseInt(volumeRange.value, 10);
      volumeValue.textContent = `${v}%`;
      this.settings = { ...this.settings, volume: v };
    });

    this.q<HTMLInputElement>("#pauseToggle").addEventListener("change", (e) => {
      this.settings = {
        ...this.settings,
        pauseOtherPlayers: (e.target as HTMLInputElement).checked,
      };
    });

    this.q<HTMLButtonElement>("#saveBtn").addEventListener("click", async () => {
      await saveSettings(this.settings);
    });

    this.q<HTMLButtonElement>("#testBtn").addEventListener("click", async () => {
      console.log("Test button clicked, triggering ceremony...");
      await triggerCeremonyNow();
    });
  }

  private async subscribeToBackendEvents(): Promise<void> {
    await onCeremonyStart(async () => {
      console.log("Ceremony start event received");
      audioPlayer.stop();
      const badge = document.getElementById("statusBadge");
      if (badge) {
        badge.textContent = "● АКТИВНА ЦЕРЕМОНІЯ";
        badge.classList.add("status-badge--active");
      }

      const settings = await getSettings();
      console.log(`Starting audio preset: ${settings.preset} with volume ${settings.volume}`);
      await audioPlayer.playPreset(settings.preset, settings.volume);
      
      console.log("Audio playback finished, notifying backend");
      await finishCeremonyNow();
    });

    await onCeremonyEnd(() => {
      console.log("Ceremony end event received");
      audioPlayer.stop();
      const badge = document.getElementById("statusBadge");
      if (badge) {
        badge.textContent = "○ ОЧІКУВАННЯ";
        badge.classList.remove("status-badge--active");
      }
    });
  }

  // ── Helpers ───────────────────────────────────────────────────────────────

  private q<T extends Element>(selector: string): T {
    const el = this.root.querySelector<T>(selector);
    if (!el) throw new Error(`Element not found: ${selector}`);
    return el;
  }
}
