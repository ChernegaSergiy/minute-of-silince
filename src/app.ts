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
import { open } from "@tauri-apps/plugin-shell";
import type { Settings, StatusSnapshot } from "./types";
import { PRESET_LABELS } from "./types";
import { t } from "./i18n";

export class App {
  private root: HTMLElement;
  private settings!: Settings;
  private cleanSettings!: Settings;
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
      this.cleanSettings = { ...this.settings };
    } catch (err) {
      console.error("Failed to load initial data from backend:", err);
      return;
    }

    this.render();
    this.bindEvents();
    await this.subscribeToBackendEvents();
    this.initOverlay();
    this.refreshStatus();
    this.startStatusPolling();
    this.updateReminderMinutesVisibility(this.settings.reminderEnabled);
  }

  private initOverlay(): void {
    onCeremonyStart(() => {
      if (this.settings.showVisualOverlay) {
        this.showOverlay();
      }
    });
    onCeremonyEnd(() => this.hideOverlay());
    
    // Initial check in case app started during ceremony
    if (this.status.ceremonyActive && this.settings.showVisualOverlay) {
      this.showOverlay();
    }
  }

  private showOverlay(): void {
    const overlay = document.getElementById("overlay");
    if (overlay) {
      overlay.classList.add("overlay--visible");
    }
  }

  private hideOverlay(): void {
    const overlay = document.getElementById("overlay");
    if (overlay) {
      overlay.classList.remove("overlay--visible");
    }
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
        if (ntpStatus.includes(t("status.ntp_disabled"))) {
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
      badge.textContent = this.status.ceremonyActive ? t("status.active") : t("status.waiting");
      if (this.status.ceremonyActive) {
        badge.classList.add("status-badge--active");
      } else {
        badge.classList.remove("status-badge--active");
      }
    }

    const skipToggle = document.getElementById("skipToggle") as HTMLInputElement;
    if (skipToggle) {
      skipToggle.checked = this.status.skipTomorrow;
    }
  }

  private startStatusPolling(): void {
    setInterval(() => this.refreshStatus(), 60000);
  }

  // Rendering

  private render(): void {
    this.root.innerHTML = `
      <div class="window">
        <header class="window__header">
          <span class="window__title">${t("header.title")}</span>
          <span class="window__version">${t("header.version", { version: import.meta.env.PACKAGE_VERSION })}</span>
        </header>

        <main class="window__body">
          <!-- Tab navigation -->
          <nav class="tabs">
            <button class="tab-btn tab-btn--active" id="settingsTabBtn">${t("tabs.settings")}</button>
            <button class="tab-btn" id="aboutTabBtn">${t("tabs.about")}</button>
          </nav>

          <!-- Settings Tab Content -->
          <div id="settingsTabContent" class="tab-content tab-content--active">
            <!-- Status badge -->
            <div class="status-badge ${this.status.ceremonyActive ? "status-badge--active" : ""}"
                 id="statusBadge">
              ${this.status.ceremonyActive ? t("status.active") : t("status.waiting")}
            </div>

            <!-- Ceremony enabled toggle -->
            <label class="control-row">
              <div class="control-row__info">
                <span class="control-row__label">${t("controls.ceremony.label")}</span>
                <span class="control-row__description">${t("controls.ceremony.description")}</span>
              </div>
              <input type="checkbox" id="ceremonyToggle" class="toggle"
                     ${this.settings.ceremonyEnabled ? "checked" : ""} />
            </label>

            <!-- Autostart toggle -->
            <label class="control-row">
              <div class="control-row__info">
                <span class="control-row__label">${t("controls.autostart.label")}</span>
                <span class="control-row__description">${t("controls.autostart.description")}</span>
              </div>
              <input type="checkbox" id="autostartToggle" class="toggle"
                     ${this.settings.autostartEnabled ? "checked" : ""} />
            </label>

            <!-- Late start grace window -->
            <div class="control-row control-row--column">
              <div class="control-row__header">
                <div class="control-row__info">
                  <span class="control-row__label">${t("controls.grace.label")}</span>
                  <span class="control-row__description">${t("controls.grace.description")}</span>
                </div>
                <span class="control-row__value" id="graceValue">${this.settings.lateStartGraceMinutes} ${t("controls.grace.unit")}</span>
              </div>
              <input type="range" id="graceRange" class="slider"
                     min="0" max="5" value="${this.settings.lateStartGraceMinutes}" />
            </div>

            <!-- Weekdays only toggle -->
            <label class="control-row">
              <div class="control-row__info">
                <span class="control-row__label">${t("controls.weekdays.label")}</span>
                <span class="control-row__description">${t("controls.weekdays.description")}</span>
              </div>
              <input type="checkbox" id="weekdaysToggle" class="toggle"
                     ${this.settings.weekdaysOnly ? "checked" : ""} />
             </label>

            <!-- System time toggle -->
            <label class="control-row">
              <div class="control-row__info">
                <span class="control-row__label">${t("controls.system_time.label")}</span>
                <span class="control-row__description">${t("controls.system_time.description")}</span>
              </div>
              <input type="checkbox" id="systemTimeToggle" class="toggle"
                     ${this.settings.systemTimeOnly ? "checked" : ""} />
            </label>

            <!-- Skip tomorrow toggle -->
            <label class="control-row">
              <div class="control-row__info">
                <span class="control-row__label">${t("controls.skip_tomorrow.label")}</span>
                <span class="control-row__description">${t("controls.skip_tomorrow.description")}</span>
              </div>
              <input type="checkbox" id="skipToggle" class="toggle"
                     ${this.status.skipTomorrow ? "checked" : ""} />
            </label>

            <!-- Reminder notification -->
            <label class="control-row">
              <div class="control-row__info">
                <span class="control-row__label">${t("controls.reminder.label")}</span>
                <span class="control-row__description">${t("controls.reminder.description")}</span>
              </div>
              <input type="checkbox" id="reminderToggle" class="toggle"
                     ${this.settings.reminderEnabled ? "checked" : ""} />
            </label>

            <!-- Reminder minutes (visible when reminder is enabled) -->
            <div class="control-row" id="reminderMinutesRow">
              <div class="control-row__info">
                <span class="control-row__label">${t("controls.reminder.in")}</span>
              </div>
              <select id="reminderSelect" class="select" style="width: 80px">
                ${this.renderReminderOptions()}
              </select>
            </div>

            <hr class="divider" />

            <!-- Audio preset -->
            <div class="control-row">
              <div class="control-row__info">
                <span class="control-row__label">${t("controls.audio_mode.label")}</span>
                <span class="control-row__description">${t("controls.audio_mode.description")}</span>
              </div>
              <select id="presetSelect" class="select">
                ${this.renderPresetOptions()}
              </select>
            </div>

            <!-- Volume -->
            <div class="control-row control-row--column">
              <div class="control-row__header">
                <div class="control-row__info">
                  <span class="control-row__label">${t("controls.volume.label")}</span>
                  <span class="control-row__description">${t("controls.volume.description")}</span>
                </div>
                <span class="control-row__value" id="volumeValue">${this.settings.volume}%</span>
              </div>
              <input type="range" id="volumeRange" class="slider"
                     min="0" max="100" value="${this.settings.volume}" />
            </div>

            <hr class="divider" />

            <!-- Volume priority toggle -->
            <label class="control-row">
              <div class="control-row__info">
                <span class="control-row__label">${t("controls.volume_priority.label")}</span>
                <span class="control-row__description">${t("controls.volume_priority.description")}</span>
              </div>
              <input type="checkbox" id="volumePriorityToggle" class="toggle"
                     ${this.settings.volumePriority ? "checked" : ""} />
            </label>

            <!-- Auto-unmute toggle -->
            <label class="control-row">
              <div class="control-row__info">
                <span class="control-row__label">${t("controls.auto_unmute.label")}</span>
                <span class="control-row__description">${t("controls.auto_unmute.description")}</span>
              </div>
              <input type="checkbox" id="autoUnmuteToggle" class="toggle"
                     ${this.settings.autoUnmute ? "checked" : ""} />
            </label>

            <!-- Pause other players -->
            <label class="control-row">
              <div class="control-row__info">
                <span class="control-row__label">${t("controls.pause.label")}</span>
                <span class="control-row__description">${t("controls.pause.description")}</span>
              </div>
              <input type="checkbox" id="pauseToggle" class="toggle"
                     ${this.settings.pauseOtherPlayers ? "checked" : ""} />
            </label>

            <!-- Visual overlay toggle -->
            <label class="control-row">
              <div class="control-row__info">
                <span class="control-row__label">${t("controls.overlay.label")}</span>
                <span class="control-row__description">${t("controls.overlay.description")}</span>
              </div>
              <input type="checkbox" id="overlayToggle" class="toggle"
                     ${this.settings.showVisualOverlay ? "checked" : ""} />
            </label>

            <!-- Meta / debug info -->
            <div class="meta">
              <span>${t("status.last_ceremony")} <span id="lastActivationValue">${this.status.lastActivation ?? "—"}</span></span>
              <div class="meta-row">
                <span>${t("status.ntp_sync")} <span id="ntpSyncValue">${this.status.lastNtpSync ?? "—"}</span></span>
                <button class="btn btn--link" id="syncNtpBtn">
                  ${t("buttons.sync")}
                </button>
              </div>
            </div>
          </div>

          <!-- About Tab Content -->
          <div id="aboutTabContent" class="tab-content">
            <img src="/logo.png" style="display: block; margin: 16px auto; max-width: 80px; height: auto;" alt="Logo" />
            <div class="meta" style="font-size: 11px; gap: 12px; margin-top: 10px;">
              <p>${t("about.description")}</p>

              <div class="meta-row" style="flex-direction: column; align-items: flex-start; gap: 4px;">
                <span>${t("about.version")} v${import.meta.env.PACKAGE_VERSION}</span>
                <span>${t("about.license")}</span>
              </div>

              <div class="meta-row" style="flex-direction: column; align-items: flex-start; gap: 4px;">
                <span>${t("about.github")}</span>
                <button class="btn btn--link" id="githubLinkBtn" style="margin: 0;">github.com/ChernegaSergiy/minute-of-silence</button>
              </div>

              <p style="opacity: 0.5; font-size: 9px; margin-top: 10px;">
                ${t("about.slava")}
              </p>
            </div>
          </div>
        </main>

        <footer class="window__footer" id="windowFooter">
          <button class="btn btn--ghost" id="testBtn">${t("buttons.test")}</button>
          <button class="btn btn--primary" id="saveBtn">${t("buttons.save")}</button>
        </footer>
      </div>
    `;
  }

  /** Build <option> list for the reminder select (0–10 min). */
  private renderReminderOptions(): string {
    const current = this.settings.reminderMinutesBefore ?? 0;
    const options: string[] = [];
    for (let m = 0; m <= 10; m++) {
      options.push(
        `<option value="${m}" ${current === m ? "selected" : ""}>${m === 0 ? t("controls.reminder.immediately") : m + " " + t("controls.grace.unit")}</option>`
      );
    }
    return options.join("");
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

  private setDirty(dirty: boolean): void {
    const saveBtn = document.getElementById("saveBtn");
    if (saveBtn) {
      if (dirty) {
        saveBtn.classList.add("btn--dirty");
        saveBtn.textContent = t("buttons.save_dirty");
      } else {
        saveBtn.classList.remove("btn--dirty");
        saveBtn.textContent = t("buttons.save");
      }
    }
  }

  private checkDirty(): void {
    const dirty = JSON.stringify(this.settings) !== JSON.stringify(this.cleanSettings);
    this.setDirty(dirty);
  }

  // Event wiring

  private bindEvents(): void {
    // Tab switching
    const settingsTabBtn = this.q<HTMLButtonElement>("#settingsTabBtn");
    const aboutTabBtn = this.q<HTMLButtonElement>("#aboutTabBtn");
    const settingsContent = this.q<HTMLElement>("#settingsTabContent");
    const aboutContent = this.q<HTMLElement>("#aboutTabContent");
    const footer = this.q<HTMLElement>("#windowFooter");

    settingsTabBtn.addEventListener("click", () => {
      settingsTabBtn.classList.add("tab-btn--active");
      aboutTabBtn.classList.remove("tab-btn--active");
      settingsContent.classList.add("tab-content--active");
      aboutContent.classList.remove("tab-content--active");
      footer.classList.remove("hidden");
    });

    aboutTabBtn.addEventListener("click", () => {
      aboutTabBtn.classList.add("tab-btn--active");
      settingsTabBtn.classList.remove("tab-btn--active");
      aboutContent.classList.add("tab-content--active");
      settingsContent.classList.remove("tab-content--active");
      footer.classList.add("hidden");
    });


    // About link
    this.q<HTMLButtonElement>("#githubLinkBtn").addEventListener("click", async () => {
      await open("https://github.com/ChernegaSergiy/minute-of-silence");
    });
 
    // Ceremony toggle
    this.q<HTMLInputElement>("#ceremonyToggle").addEventListener("change", (e) => {
      this.settings = {
        ...this.settings,
        ceremonyEnabled: (e.target as HTMLInputElement).checked,
      };
      this.checkDirty();
    });
 
    // Autostart toggle
    this.q<HTMLInputElement>("#autostartToggle").addEventListener("change", (e) => {
      this.settings = {
        ...this.settings,
        autostartEnabled: (e.target as HTMLInputElement).checked,
      };
      this.checkDirty();
    });
 
    // Grace window slider
    const graceRange = this.q<HTMLInputElement>("#graceRange");
    const graceValue = this.q<HTMLElement>("#graceValue");
    graceRange.addEventListener("input", () => {
      const v = parseInt(graceRange.value, 10);
      graceValue.textContent = `${v} ${t("controls.grace.unit")}`;
      this.settings = { ...this.settings, lateStartGraceMinutes: v };
      this.checkDirty();
    });
 
    // Weekdays toggle
    this.q<HTMLInputElement>("#weekdaysToggle").addEventListener("change", (e) => {
      this.settings = {
        ...this.settings,
        weekdaysOnly: (e.target as HTMLInputElement).checked,
      };
      this.checkDirty();
    });
 
    // System time toggle
    this.q<HTMLInputElement>("#systemTimeToggle").addEventListener("change", (e) => {
      this.settings = {
        ...this.settings,
        systemTimeOnly: (e.target as HTMLInputElement).checked,
      };
      this.checkDirty();
    });
 
    // Skip toggle (immediate, no save needed)
    this.q<HTMLInputElement>("#skipToggle").addEventListener("change", (e) => {
      if ((e.target as HTMLInputElement).checked) {
        skipNext();
      } else {
        unskipNext();
      }
    });
 
    // Reminder toggle
    this.q<HTMLInputElement>("#reminderToggle").addEventListener("change", (e) => {
      const checked = (e.target as HTMLInputElement).checked;
      this.settings = { ...this.settings, reminderEnabled: checked };
      this.updateReminderMinutesVisibility(checked);
      this.checkDirty();
    });

    // Reminder select
    this.q<HTMLSelectElement>("#reminderSelect").addEventListener("change", (e) => {
      const v = parseInt((e.target as HTMLSelectElement).value, 10);
      this.settings = { ...this.settings, reminderMinutesBefore: v };
      this.checkDirty();
    });
 
    // Preset select
    this.q<HTMLSelectElement>("#presetSelect").addEventListener("change", (e) => {
      this.settings = {
        ...this.settings,
        preset: (e.target as HTMLSelectElement).value as Settings["preset"],
      };
      this.checkDirty();
    });
 
    // Volume slider
    const volumeRange = this.q<HTMLInputElement>("#volumeRange");
    const volumeValue = this.q<HTMLElement>("#volumeValue");
    volumeRange.addEventListener("input", () => {
      const v = parseInt(volumeRange.value, 10);
      volumeValue.textContent = `${v}%`;
      this.settings = { ...this.settings, volume: v };
      this.checkDirty();
    });
 
    // Volume priority toggle
    this.q<HTMLInputElement>("#volumePriorityToggle").addEventListener("change", (e) => {
      this.settings = {
        ...this.settings,
        volumePriority: (e.target as HTMLInputElement).checked,
      };
      this.checkDirty();
    });
 
    // Auto-unmute toggle
    this.q<HTMLInputElement>("#autoUnmuteToggle").addEventListener("change", (e) => {
      this.settings = {
        ...this.settings,
        autoUnmute: (e.target as HTMLInputElement).checked,
      };
      this.checkDirty();
    });
 
    // Pause other players toggle
    this.q<HTMLInputElement>("#pauseToggle").addEventListener("change", (e) => {
      this.settings = {
        ...this.settings,
        pauseOtherPlayers: (e.target as HTMLInputElement).checked,
      };
      this.checkDirty();
    });
 
    // Visual overlay toggle
    this.q<HTMLInputElement>("#overlayToggle").addEventListener("change", (e) => {
      this.settings = {
        ...this.settings,
        showVisualOverlay: (e.target as HTMLInputElement).checked,
      };
      this.checkDirty();
    });
 
    // Save button
    this.q<HTMLButtonElement>("#saveBtn").addEventListener("click", async () => {
      await saveSettings(this.settings);
      this.cleanSettings = { ...this.settings };
      await this.refreshStatus();
      this.setDirty(false);
    });
 
    // Test button
    this.q<HTMLButtonElement>("#testBtn").addEventListener("click", async () => {
      console.log("Test button clicked, triggering ceremony...");
      await triggerCeremonyNow();
    });
 
    // Manual NTP sync button
    this.q<HTMLButtonElement>("#syncNtpBtn").addEventListener("click", async (e) => {
      const btn = e.target as HTMLButtonElement;
      const ntpEl = document.getElementById("ntpSyncValue");
 
      btn.disabled = true;
      if (ntpEl) ntpEl.textContent = t("status.ntp_syncing");
 
      try {
        const { syncNtpNow } = await import("./api");
        this.status = await syncNtpNow();
        this.updateStatusUI();
      } catch (err) {
        console.error("Manual NTP sync failed:", err);
        if (ntpEl) ntpEl.textContent = t("status.ntp_error");
      } finally {
        btn.disabled = false;
      }
    });
 
    // Disable default context menu globally for a native app feel
    window.addEventListener("contextmenu", (e) => e.preventDefault());
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
 
    await listen("status-updated", () => {
      console.log("Status updated event received");
      this.refreshStatus();
    });
  }

  // Helpers

  private q<T extends Element>(selector: string): T {
    const el = this.root.querySelector<T>(selector);
    if (!el) throw new Error(`Element not found: ${selector}`);
    return el;
  }

  private updateReminderMinutesVisibility(enabled: boolean): void {
    const row = document.getElementById("reminderMinutesRow");
    if (row) {
      row.classList.toggle("hidden", !enabled);
    }
  }
}
