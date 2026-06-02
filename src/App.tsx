import { Suspense, lazy, useCallback, useEffect, useRef, useState } from "react";
import {
  Button,
  FluentProvider,
  NavDrawer,
  NavDrawerBody,
  NavItem,
  Spinner,
  makeStyles,
  shorthands,
  tokens,
  webDarkTheme,
  webLightTheme,
} from "@fluentui/react-components";
import {
  DocumentBulletList20Regular,
  Info20Regular,
  Play20Regular,
  Save20Regular,
  Settings20Regular,
  CalendarMonth20Regular,
} from "@fluentui/react-icons";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { getVersion } from "@tauri-apps/api/app";
import { listen } from "@tauri-apps/api/event";
import {
  bringWindowToFront,
  getSettings,
  getPersonalDates,
  getStatus,
  onCeremonyEnd,
  onCeremonyStart,
  saveSettings,
  syncNtpNow,
  triggerCeremonyNow,
} from "./utils/api";
import { DEFAULT_SETTINGS, type PersonalDate, type Settings, type StatusSnapshot } from "./types";
import { t } from "./utils/i18n";
import AboutTab from "./components/AboutTab";
import Overlay from "./components/Overlay";
import SettingsTab from "./components/SettingsTab";
import PersonalDatesTab from "./components/PersonalDatesTab";
import UpdateDialog, { type UpdateInfo } from "./components/UpdateDialog";
import { useIdle } from "./hooks/useIdle";

const ChangelogTab = lazy(() => import("./components/ChangelogTab"));

const DEFAULT_STATUS: StatusSnapshot = {
  ceremonyActive: false,
  skipTomorrow: false,
  lastActivation: null,
  lastNtpSync: null,
};

const useStyles = makeStyles({
  layout: {
    height: "100%",
    display: "flex",
    flexDirection: "column",
  },
  body: {
    display: "flex",
    flex: 1,
    ...shorthands.overflow("hidden"),
  },
  content: {
    flex: 1,
    display: "flex",
    flexDirection: "column",
    ...shorthands.overflow("hidden"),
  },
  scroll: {
    flex: 1,
    overflowY: "auto",
    padding: tokens.spacingVerticalL,
  },
  buttonBar: {
    display: "flex",
    gap: tokens.spacingHorizontalM,
    padding: tokens.spacingVerticalL,
    ...shorthands.borderTop("1px", "solid", tokens.colorNeutralStroke2),
  },
  changelogSpinner: {
    display: "flex",
    justifyContent: "center",
    paddingTop: tokens.spacingVerticalL,
  },
});

export default function App() {
  const styles = useStyles();
  const [selectedNav, setSelectedNav] = useState<string>("settings");
  const [prefersDark, setPrefersDark] = useState(() =>
    typeof window !== "undefined"
      ? window.matchMedia("(prefers-color-scheme: dark)").matches
      : true
  );
  const [settings, setSettings] = useState<Settings>(DEFAULT_SETTINGS);
  const [personalDates, setPersonalDates] = useState<PersonalDate[]>([]);
  const [cleanSettings, setCleanSettings] = useState<string>("");
  const [status, setStatus] = useState<StatusSnapshot>(DEFAULT_STATUS);
  const [version, setVersion] = useState("...");
  const [showOverlay, setShowOverlay] = useState(false);
  const [ceremonyDurationMs, setCeremonyDurationMs] = useState<number | undefined>(undefined);
  const [volumeValue, setVolumeValue] = useState(80);
  const [syncing, setSyncing] = useState(false);
  const [hydrated, setHydrated] = useState(false);
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);
  const [showUpdateDialog, setShowUpdateDialog] = useState(false);
  const [updateDismissed, setUpdateDismissed] = useState(false);
  const isIdle = useIdle(15000); // 15 seconds idle timeout
  const initRef = useRef(false);

  const isDirty = hydrated && JSON.stringify(settings) !== cleanSettings;
  const effectiveDark = (settings.useSystemTheme ?? true) ? prefersDark : settings.uiTheme === "dark";

  useEffect(() => {
    if (initRef.current) return;
    initRef.current = true;

    (async () => {
      try {
        const [s, dates, st, v] = await Promise.all([getSettings(), getPersonalDates(), getStatus(), getVersion()]);
        setSettings(s);
        setCleanSettings(JSON.stringify(s));
        setPersonalDates(dates);
        setStatus(st);
        setVolumeValue(s.volume);
        setVersion(v);
        await getCurrentWindow().setTitle(t("app.title"));

        // Check for updates on startup
        const { invoke } = await import("@tauri-apps/api/core");
        const update = await invoke<UpdateInfo | null>("check_for_updates");
        if (update) {
          setUpdateInfo(update);
        }
      } catch (error) {
        console.error(error);
      } finally {
        setHydrated(true);
      }
    })();
  }, []);

  useEffect(() => {
    const refresh = async () => {
      try {
        setStatus(await getStatus());
      } catch {
        // ignore transient status refresh errors
      }
    };

    const unlisteners: (() => void)[] = [];
    (async () => {
      unlisteners.push(
        await onCeremonyStart((_p) => refresh()),
        await onCeremonyEnd(refresh),
        await listen("ntp-synced", refresh),
        await listen("status-updated", refresh),
        await listen<UpdateInfo>("update-available", (event) => {
          setUpdateInfo(event.payload);
        })
      );
      setInterval(refresh, 60000);
    })();

    return () => unlisteners.forEach((u) => u());
  }, []);

  useEffect(() => {
    if (!settings.showVisualOverlay) {
      setShowOverlay(false);
      return;
    }

    let cancelled = false;
    const unlisteners: (() => void)[] = [];

    (async () => {
      unlisteners.push(
        await onCeremonyStart(async (payload) => {
          if (cancelled) return;
          setShowOverlay(true);
          setCeremonyDurationMs(payload?.duration_ms);
          try {
            await bringWindowToFront();
          } catch {
            // ignore focus errors
          }
        }),
        await onCeremonyEnd(() => {
          if (!cancelled) {
            setShowOverlay(false);
            setCeremonyDurationMs(undefined);
          }
        })
      );

      if (!cancelled && status.ceremonyActive) {
        setShowOverlay(true);
      }
    })();

    return () => {
      cancelled = true;
      unlisteners.forEach((u) => u());
    };
  }, [settings.showVisualOverlay, status.ceremonyActive]);

  useEffect(() => {
    const media = window.matchMedia("(prefers-color-scheme: dark)");

    const updateTheme = (event: MediaQueryListEvent) => {
      setPrefersDark(event.matches);
    };

    setPrefersDark(media.matches);
    media.addEventListener("change", updateTheme);

    return () => media.removeEventListener("change", updateTheme);
  }, []);

  // Trigger the update dialog when user is idle
  useEffect(() => {
    if (!updateInfo || updateDismissed || showUpdateDialog) return;

    if (isIdle && document.hasFocus()) {
      setShowUpdateDialog(true);
    }
  }, [updateInfo, updateDismissed, showUpdateDialog, isIdle]);

  const updateSetting = useCallback(<K extends keyof Settings>(key: K, value: Settings[K]) => {
    setSettings((prev) => ({ ...prev, [key]: value }));
  }, []);

  const handleSave = useCallback(async () => {
    await saveSettings(settings);
    setCleanSettings(JSON.stringify(settings));
  }, [settings]);

  const handleSyncNtp = useCallback(async () => {
    setSyncing(true);
    try {
      setStatus(await syncNtpNow());
    } finally {
      setSyncing(false);
    }
  }, []);

  return (
    <>
      <FluentProvider
        theme={effectiveDark ? webDarkTheme : webLightTheme}
        style={{ height: "100%" }}
      >
        <div className={styles.layout}>
          <div className={styles.body}>
            <NavDrawer
              selectedValue={selectedNav}
              onNavItemSelect={(_, data) => setSelectedNav(data.value as string)}
              open
              type="inline"
              density="small"
            >
              <NavDrawerBody key={selectedNav}>
                <NavItem value="settings" icon={<Settings20Regular />}>
                  {t("tabs.settings")}
                </NavItem>
                <NavItem value="personal_dates" icon={<CalendarMonth20Regular />}>
                  {t("tabs.personal_dates")}
                </NavItem>
                <NavItem value="about" icon={<Info20Regular />}>
                  {t("tabs.about")}
                </NavItem>
                <NavItem value="changelog" icon={<DocumentBulletList20Regular />}>
                  {t("tabs.changelog")}
                </NavItem>
              </NavDrawerBody>
            </NavDrawer>

            <div className={styles.content}>
              {selectedNav === "changelog" ? (
                <Suspense
                  fallback={
                    <div className={styles.changelogSpinner}>
                      <Spinner size="tiny" />
                    </div>
                  }
                >
                  <ChangelogTab />
                </Suspense>
              ) : (
                <div className={styles.scroll}>
                  {selectedNav === "settings" ? (
                    <SettingsTab
                      settings={settings}
                      status={status}
                      volumeValue={volumeValue}
                      syncing={syncing}
                      onUpdateSetting={updateSetting}
                      onVolumeChange={setVolumeValue}
                      onSyncNtp={handleSyncNtp}
                    />
                  ) : selectedNav === "personal_dates" ? (
                    <PersonalDatesTab
                      personalDates={personalDates}
                      onPersonalDatesChange={setPersonalDates}
                    />
                  ) : (
                    <AboutTab version={version} />
                  )}
                </div>
              )}
            </div>
          </div>

          {selectedNav === "settings" && (
            <div className={styles.buttonBar}>
              <Button
                icon={<Play20Regular />}
                appearance="secondary"
                onClick={triggerCeremonyNow}
                disabled={!hydrated}
              >
                {t("buttons.test")}
              </Button>
              <Button
                icon={<Save20Regular />}
                appearance="primary"
                disabled={!hydrated || !isDirty}
                onClick={handleSave}
              >
                {isDirty ? t("buttons.save_dirty") : t("buttons.save")}
              </Button>
            </div>
          )}
        </div>
        <UpdateDialog
          updateInfo={showUpdateDialog ? updateInfo : null}
          onClose={() => {
            setShowUpdateDialog(false);
            setUpdateDismissed(true);
          }}
        />
      </FluentProvider>

      <Overlay
        show={showOverlay}
        durationSeconds={ceremonyDurationMs ? ceremonyDurationMs / 1000 : undefined}
        personalDates={personalDates}
      />
    </>
  );
}
