/**
 * Root application controller.
 *
 * Responsibilities:
 *  - Bootstrap the UI from the DOM template in `index.html`.
 *  - Load initial settings and status from the backend.
 *  - Wire up all user-interaction event handlers.
 *  - Subscribe to ceremony events.
 */

import {
  getSettings,
  getStatus,
  saveSettings,
  skipNext,
  unskipNext,
  triggerCeremonyNow,
  onCeremonyStart,
  onCeremonyEnd,
} from "./api";

import { listen } from "@tauri-apps/api/event";
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
    this.refreshStatus();
    this.startStatusPolling();
  }

  private async refreshStatus(): Promise<void> {
    try {
      this.status = await getStatus();
      this.updateStatusUI();
    } catch (err) {
      console.error("Failed to refresh status:", err);
    }
  }

  private updateStatusUI(): void {
    const ntpEl = document.getElementById("ntpSyncValue");
    const syncBtn = document.getElementById("syncNtpBtn");
    
    if (ntpEl) {
      const ntpStatus = this.status.lastNtpSync ?? "—";
      ntpEl.textContent = ntpStatus;
      
      // Hide sync button if NTP is disabled in the backend status
      if (syncBtn) {
        if (ntpStatus.includes("Вимкнено")) {
          syncBtn.classList.add("hidden");
        } else {
          syncBtn.classList.remove("hidden");
        }
      }
    }
    const ceremonyEl = document.getElementById("lastActivationValue");
    if (ceremonyEl) {
      ceremonyEl.textContent = this.status.lastActivation ?? "—";
    }
    const badge = document.getElementById("statusBadge");
    if (badge) {
      badge.textContent = this.status.ceremonyActive ? "● АКТИВНА ЦЕРЕМОНІЯ" : "○ ОЧІКУВАННЯ";
      if (this.status.ceremonyActive) {
        badge.classList.add("status-badge--active");
      } else {
        badge.classList.remove("status-badge--active");
      }
    }
  }

  private startStatusPolling(): void {
    setInterval(() => this.refreshStatus(), 60000);
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

          <!-- System time toggle -->
          <label class="control-row">
            <span class="control-row__label">Системний час</span>
            <input type="checkbox" id="systemTimeToggle" class="toggle"
                   ${this.settings.systemTimeOnly ? "checked" : ""} />
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

          <!-- Volume priority toggle -->
          <label class="control-row">
            <span class="control-row__label">Пріоритет гучності</span>
            <input type="checkbox" id="volumePriorityToggle" class="toggle"
                   ${this.settings.volumePriority ? "checked" : ""} />
          </label>

          <!-- Pause other players -->
          <label class="control-row">
            <span class="control-row__label">Пауза інших плеєрів</span>
            <input type="checkbox" id="pauseToggle" class="toggle"
                   ${this.settings.pauseOtherPlayers ? "checked" : ""} />
          </label>


          <!-- Meta / debug info -->
          <div class="meta">
            <span>Остання церемонія: <span id="lastActivationValue">${this.status.lastActivation ?? "—"}</span></span>
            <div class="meta-row">
              <span>Синхронізація NTP: <span id="ntpSyncValue">${this.status.lastNtpSync ?? "—"}</span></span>
              <button class="btn btn--link" id="syncNtpBtn">
                СИНХРОНІЗУВАТИ
              </button>
            </div>
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

    this.q<HTMLInputElement>("#systemTimeToggle").addEventListener(
      "change",
      (e) => {
        this.settings = {
          ...this.settings,
          systemTimeOnly: (e.target as HTMLInputElement).checked,
        };
      }
    );

    this.q<HTMLInputElement>("#volumePriorityToggle").addEventListener(
      "change",
      (e) => {
        this.settings = {
          ...this.settings,
          volumePriority: (e.target as HTMLInputElement).checked,
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
      await this.refreshStatus(); // Immediately update status UI (NTP, activation info, etc)
    });

    this.q<HTMLButtonElement>("#testBtn").addEventListener("click", async () => {
      console.log("Test button clicked, triggering ceremony...");
      await triggerCeremonyNow();
    });

    this.q<HTMLButtonElement>("#syncNtpBtn").addEventListener("click", async (e) => {
      const btn = e.target as HTMLButtonElement;
      const ntpEl = document.getElementById("ntpSyncValue");
      
      btn.disabled = true;
      if (ntpEl) ntpEl.textContent = "Синхронізація...";
      
      try {
        const { syncNtpNow } = await import("./api");
        this.status = await syncNtpNow();
        this.updateStatusUI();
      } catch (err) {
        console.error("Manual NTP sync failed:", err);
        if (ntpEl) ntpEl.textContent = "Помилка";
      } finally {
        btn.disabled = false;
      }
    });
  }

  private async subscribeToBackendEvents(): Promise<void> {
    await onCeremonyStart(async () => {
      console.log("Ceremony start event received");
      this.refreshStatus();
    });

    await onCeremonyEnd(() => {
      console.log("Ceremony end event received");
      this.refreshStatus();
    });

    await listen("ntp-synced", () => {
      console.log("NTP synced event received");
      this.refreshStatus();
    });
  }

  // ── Helpers ───────────────────────────────────────────────────────────────

  private q<T extends Element>(selector: string): T {
    const el = this.root.querySelector<T>(selector);
    if (!el) throw new Error(`Element not found: ${selector}`);
    return el;
  }
}
