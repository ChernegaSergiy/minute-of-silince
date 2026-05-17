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
} from "@fluentui/react-icons";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { getVersion } from "@tauri-apps/api/app";
import { listen } from "@tauri-apps/api/event";
import {
  bringWindowToFront,
  getSettings,
  getStatus,
  onCeremonyEnd,
  onCeremonyStart,
  saveSettings,
  syncNtpNow,
  triggerCeremonyNow,
} from "./api";
import { DEFAULT_SETTINGS, type Settings, type StatusSnapshot } from "./types";
import { t } from "./i18n";
import AboutTab from "./AboutTab";
import Overlay from "./Overlay";
import SettingsTab from "./SettingsTab";

const ChangelogTab = lazy(() => import("./ChangelogTab"));

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
  const [cleanSettings, setCleanSettings] = useState<string>("");
  const [status, setStatus] = useState<StatusSnapshot>(DEFAULT_STATUS);
  const [version, setVersion] = useState("...");
  const [showOverlay, setShowOverlay] = useState(false);
  const [volumeValue, setVolumeValue] = useState(80);
  const [syncing, setSyncing] = useState(false);
  const [hydrated, setHydrated] = useState(false);
  const initRef = useRef(false);

  const isDirty = hydrated && JSON.stringify(settings) !== cleanSettings;

  useEffect(() => {
    if (initRef.current) return;
    initRef.current = true;

    (async () => {
      try {
        const [s, st, v] = await Promise.all([getSettings(), getStatus(), getVersion()]);
        setSettings(s);
        setCleanSettings(JSON.stringify(s));
        setStatus(st);
        setVolumeValue(s.volume);
        setVersion(v);
        await getCurrentWindow().setTitle(t("app.title"));
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
        await onCeremonyStart(refresh),
        await onCeremonyEnd(refresh),
        await listen("ntp-synced", refresh),
        await listen("status-updated", refresh)
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
        await onCeremonyStart(async () => {
          if (cancelled) return;
          setShowOverlay(true);
          try {
            await bringWindowToFront();
          } catch {
            // ignore focus errors
          }
        }),
        await onCeremonyEnd(() => {
          if (!cancelled) setShowOverlay(false);
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
        theme={prefersDark ? webDarkTheme : webLightTheme}
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
              <NavDrawerBody>
                <NavItem value="settings" icon={<Settings20Regular />}>
                  {t("tabs.settings")}
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
      </FluentProvider>

      <Overlay show={showOverlay} />
    </>
  );
}
