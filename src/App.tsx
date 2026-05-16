import { useEffect, useState, useCallback, useRef } from "react";
import {
  FluentProvider,
  webDarkTheme,
  makeStyles,
  shorthands,
  tokens,
  NavDrawer,
  NavDrawerBody,
  NavItem,
  Switch,
  Button,
  Slider,
  Text,
  Dropdown,
  Option,
  Card,
  CardHeader,
  Divider,
  Link,
} from "@fluentui/react-components";
import {
  Settings20Regular,
  Info20Regular,
  Save20Regular,
  Play20Regular,
  ArrowSync20Regular,
  DocumentBulletList20Regular,
} from "@fluentui/react-icons";
import {
  getSettings,
  saveSettings,
  getStatus,
  skipNext,
  unskipNext,
  syncNtpNow,
  triggerCeremonyNow,
  onCeremonyStart,
  onCeremonyEnd,
  bringWindowToFront,
} from "./api";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { getVersion } from "@tauri-apps/api/app";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-shell";
import type { Settings, StatusSnapshot, AudioPreset, AnnouncementVoice, AnthemVoice } from "./types";
import { t } from "./i18n";
import changelogMd from "../CHANGELOG.md?raw";
import Markdown from "react-markdown";
import Overlay from "./Overlay";

const presets: AudioPreset[] = [
  "voice_metronome", "metronome_only", "voice_silence_bell",
  "voice_silence", "voice_metronome_anthem", "voice_metronome_ending",
  "metronome_anthem", "bell_silence_bell", "bell_metronome_bell", "silence",
];

interface ChangelogCategory {
  name: string;
  items: string[];
}

interface ChangelogVersion {
  version: string;
  date: string;
  categories: ChangelogCategory[];
}

function parseChangelog(md: string): ChangelogVersion[] {
  const versions: ChangelogVersion[] = [];
  const sections = md.split(/\n(?=## \[)/);
  for (const section of sections) {
    const headerMatch = section.match(/^## \[([\d.]+)]\s*-\s*(.+)$/m);
    if (!headerMatch) continue;
    const version = headerMatch[1];
    const date = headerMatch[2].trim();
    const categories: ChangelogCategory[] = [];
    const catSections = section.split(/\n(?=### )/);
    for (const catSection of catSections) {
      const catMatch = catSection.match(/^### (.+)$/m);
      if (!catMatch) continue;
      const name = catMatch[1];
      const items = catSection
        .split("\n")
        .filter((l) => l.startsWith("- "))
        .map((l) => l.replace(/^- /, "").trim());
      if (items.length > 0) {
        categories.push({ name, items });
      }
    }
    versions.push({ version, date, categories });
  }
  return versions;
}

const changelogVersions = parseChangelog(changelogMd);

const announcementVoiceLabels: Record<string, string> = {
  bohdan_hdal: "Богдан Хдаль",
  sonia_sotnyk: "Соня Сотник",
  dania_khomutovskyi: "Даня Хомутовський",
  air_alert: "Повітряна тривога",
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
  card: {
    marginBottom: tokens.spacingVerticalM,
  },
  statusCard: {
    marginBottom: tokens.spacingVerticalM,
  },
  statusValue: {
    fontSize: tokens.fontSizeBase600,
    fontWeight: tokens.fontWeightSemibold,
    marginTop: tokens.spacingVerticalXS,
  },
  switchRow: {
    display: "flex",
    justifyContent: "space-between",
    alignItems: "center",
    paddingTop: tokens.spacingVerticalSNudge,
    paddingBottom: tokens.spacingVerticalSNudge,
  },
  switchLabel: {
    flex: 1,
    marginRight: tokens.spacingHorizontalM,
    minWidth: 0,
  },
  switchDesc: {
    fontSize: tokens.fontSizeBase100,
    color: tokens.colorNeutralForeground3,
    marginTop: tokens.spacingVerticalXXS,
  },
  selectRow: {
    marginBottom: tokens.spacingVerticalM,
  },
  selectLabel: {
    fontSize: tokens.fontSizeBase200,
    marginBottom: tokens.spacingVerticalXS,
  },
  dropdown: {
    flex: 1,
  },
  volumeRow: {
    display: "flex",
    alignItems: "center",
    gap: tokens.spacingHorizontalM,
  },
  volumeSlider: {
    flex: 1,
  },
  volumeValue: {
    fontSize: tokens.fontSizeBase200,
    minWidth: "45px",
    textAlign: "right",
  },
  buttonBar: {
    display: "flex",
    gap: tokens.spacingHorizontalM,
    padding: tokens.spacingVerticalL,
    ...shorthands.borderTop("1px", "solid", tokens.colorNeutralStroke2),
  },
  aboutContent: {
    textAlign: "center",
    padding: tokens.spacingVerticalXXL,
  },
  aboutLogo: {
    width: "96px",
    borderRadius: tokens.borderRadiusMedium,
  },
  aboutTitle: {
    fontSize: tokens.fontSizeBase500,
    fontWeight: tokens.fontWeightSemibold,
    marginTop: tokens.spacingVerticalL,
  },
  aboutVersion: {
    fontSize: tokens.fontSizeBase200,
    color: tokens.colorNeutralForeground3,
    marginTop: tokens.spacingVerticalS,
  },
  aboutDesc: {
    fontSize: tokens.fontSizeBase200,
    marginTop: tokens.spacingVerticalL,
    color: tokens.colorNeutralForeground2,
  },
  aboutLinks: {
    marginTop: tokens.spacingVerticalXXL,
    display: "flex",
    flexDirection: "column",
    gap: tokens.spacingVerticalS,
  },
  aboutLicense: {
    fontSize: tokens.fontSizeBase100,
    color: tokens.colorNeutralForeground3,
    marginTop: tokens.spacingVerticalXXL,
  },
  infoRow: {
    fontSize: tokens.fontSizeBase100,
    color: tokens.colorNeutralForeground3,
  },
  ntpRow: {
    marginTop: tokens.spacingVerticalS,
    display: "flex",
    alignItems: "center",
    gap: tokens.spacingHorizontalS,
  },
  changelogCategory: {
    marginTop: tokens.spacingVerticalS,
  },
  changelogCatHeader: {
    textTransform: "uppercase",
    letterSpacing: "0.1em",
    marginBottom: tokens.spacingVerticalXXS,
  },
  changelogList: {
    margin: 0,
    paddingLeft: tokens.spacingHorizontalL,
  },
  changelogItem: {
    marginBottom: tokens.spacingVerticalXXS,
    lineHeight: tokens.lineHeightBase200,
  },
  changelogCode: {
    fontSize: tokens.fontSizeBase200,
    backgroundColor: tokens.colorNeutralBackground5,
    padding: `0 ${tokens.spacingHorizontalXXS}`,
    borderRadius: tokens.borderRadiusSmall,
  },
  changelogContent: {
    padding: tokens.spacingVerticalL,
  },
});

function SwitchRow({
  id,
  label,
  desc,
  checked,
  onChange,
}: {
  id: string;
  label: string;
  desc: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
}) {
  const styles = useStyles();
  return (
    <div className={styles.switchRow}>
      <div className={styles.switchLabel}>
        <Text size={200}>{label}</Text>
        <div className={styles.switchDesc}>{desc}</div>
      </div>
      <Switch
        id={id}
        checked={checked}
        onChange={(_, data) => onChange(data.checked as boolean)}
      />
    </div>
  );
}

export default function App() {
  const styles = useStyles();
  const [selectedNav, setSelectedNav] = useState<string>("settings");
  const [settings, setSettings] = useState<Settings | null>(null);
  const [cleanSettings, setCleanSettings] = useState<string>("");
  const [status, setStatus] = useState<StatusSnapshot | null>(null);
  const [version, setVersion] = useState("...");
  const [showOverlay, setShowOverlay] = useState(false);
  const [volumeValue, setVolumeValue] = useState(80);
  const [syncing, setSyncing] = useState(false);
  const initRef = useRef(false);

  const isDirty =
    settings !== null && JSON.stringify(settings) !== cleanSettings;

  useEffect(() => {
    if (initRef.current) return;
    initRef.current = true;
    (async () => {
      try {
        const [s, st, v] = await Promise.all([
          getSettings(),
          getStatus(),
          getVersion(),
        ]);
        setSettings(s);
        setCleanSettings(JSON.stringify(s));
        setStatus(st);
        setVolumeValue(s.volume);
        setVersion(v);
        await getCurrentWindow().setTitle(t("app.title"));
      } catch (e) {
        console.error(e);
      }
    })();
  }, []);

  useEffect(() => {
    const refresh = async () => {
      try {
        setStatus(await getStatus());
      } catch (e) {}
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
    if (!settings) return;
    let cancelled = false;
    const unlisteners: (() => void)[] = [];

    (async () => {
      unlisteners.push(
        await onCeremonyStart(async () => {
          if (cancelled) return;
          if (settings.showVisualOverlay) {
            setShowOverlay(true);
            try {
              await bringWindowToFront();
            } catch {}
          }
        }),
        await onCeremonyEnd(() => {
          if (!cancelled) setShowOverlay(false);
        })
      );
      if (!cancelled && status?.ceremonyActive && settings.showVisualOverlay) {
        setShowOverlay(true);
      }
    })();

    return () => {
      cancelled = true;
      unlisteners.forEach((u) => u());
    };
  }, [settings, status?.ceremonyActive]);

  const updateSetting = useCallback(
    <K extends keyof Settings>(key: K, value: Settings[K]) => {
      setSettings((prev) => (prev ? { ...prev, [key]: value } : prev));
    },
    []
  );

  const handleSave = useCallback(async () => {
    if (!settings) return;
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

  if (!settings || !status) return null;

  return (
    <>
      <FluentProvider theme={webDarkTheme} style={{ height: "100%" }}>
      <div className={styles.layout}>
        <div className={styles.body}>
          <NavDrawer
            selectedValue={selectedNav}
            onNavItemSelect={(_, data) => setSelectedNav(data.value as string)}
            open={true}
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
            <div className={styles.scroll}>
              {selectedNav === "settings" ? (
                <>
                  <Card className={styles.card}>
                    <Text size={100} weight="semibold" block>
                      ОСНОВНІ
                    </Text>
                    <SwitchRow
                      id="ceremonyToggle"
                      label={t("controls.ceremony.label")}
                      desc={t("controls.ceremony.description")}
                      checked={settings.ceremonyEnabled}
                      onChange={(v) => updateSetting("ceremonyEnabled", v)}
                    />
                    <Divider />
                    <SwitchRow
                      id="autostartToggle"
                      label={t("controls.autostart.label")}
                      desc={t("controls.autostart.description")}
                      checked={settings.autostartEnabled}
                      onChange={(v) => updateSetting("autostartEnabled", v)}
                    />
                    <Divider />
                    <SwitchRow
                      id="weekdaysToggle"
                      label={t("controls.weekdays.label")}
                      desc={t("controls.weekdays.description")}
                      checked={settings.weekdaysOnly}
                      onChange={(v) => updateSetting("weekdaysOnly", v)}
                    />
                    <Divider />
                    <div>
                      <div className={styles.selectLabel}>
                        {t("controls.grace.label")}
                      </div>
                      <div className={styles.volumeRow}>
                        <Slider
                          className={styles.volumeSlider}
                          min={0}
                          max={5}
                          value={settings.lateStartGraceMinutes}
                          onChange={(_, data) =>
                            updateSetting("lateStartGraceMinutes", data.value)
                          }
                        />
                        <Text size={200} className={styles.volumeValue}>
                          {settings.lateStartGraceMinutes}{t("controls.grace.unit")}
                        </Text>
                      </div>
                    </div>
                    <Divider />
                    <SwitchRow
                      id="systemTimeToggle"
                      label={t("controls.system_time.label")}
                      desc={t("controls.system_time.description")}
                      checked={settings.systemTimeOnly}
                      onChange={(v) => updateSetting("systemTimeOnly", v)}
                    />
                    <Divider />
                    <SwitchRow
                      id="skipToggle"
                      label={t("controls.skip_tomorrow.label")}
                      desc={t("controls.skip_tomorrow.description")}
                      checked={status.skipTomorrow}
                      onChange={(v) => {
                        if (v) skipNext();
                        else unskipNext();
                      }}
                    />
                  </Card>

                  <Card className={styles.card}>
                    <Text size={100} weight="semibold" block>
                      {t("controls.reminder.label")}
                    </Text>
                    <SwitchRow
                      id="reminderToggle"
                      label={t("controls.reminder.label")}
                      desc={t("controls.reminder.description")}
                      checked={settings.reminderEnabled}
                      onChange={(v) => updateSetting("reminderEnabled", v)}
                    />
                    <Divider />
                    {settings.reminderEnabled && (
                      <>
                        <div className={styles.volumeRow}>
                          <Dropdown
                            className={styles.dropdown}
                            value={settings.reminderMinutesBefore === 0 ? t("controls.reminder.immediately") : `${t("controls.reminder.in")} ${settings.reminderMinutesBefore} ${t("controls.grace.unit")}`}
                            selectedOptions={[String(settings.reminderMinutesBefore)]}
                            onOptionSelect={(_, data) =>
                              updateSetting("reminderMinutesBefore", Number(data.optionValue))
                            }
                          >
                            <Option value="0" text={t("controls.reminder.immediately")}>{t("controls.reminder.immediately")}</Option>
                            {[1, 2, 3, 4, 5, 6, 7, 8, 9, 10].map((n) => (
                              <Option key={n} value={String(n)} text={`${t("controls.reminder.in")} ${n} ${t("controls.grace.unit")}`}>
                                {t("controls.reminder.in")} {n} {t("controls.grace.unit")}
                              </Option>
                            ))}
                          </Dropdown>
                        </div>
                        <Divider />
                      </>
                    )}
                    <SwitchRow
                      id="overlayToggle"
                      label={t("controls.overlay.label")}
                      desc={t("controls.overlay.description")}
                      checked={settings.showVisualOverlay}
                      onChange={(v) => updateSetting("showVisualOverlay", v)}
                    />
                    <Divider />
                    <SwitchRow
                      id="flagAnimationToggle"
                      label={t("controls.flag_animation.label")}
                      desc={t("controls.flag_animation.description")}
                      checked={settings.showFlagAnimation}
                      onChange={(v) => updateSetting("showFlagAnimation", v)}
                    />
                  </Card>

                  <Card className={styles.card}>
                    <Text size={100} weight="semibold" block>
                      {t("controls.audio_mode.label")}
                    </Text>
                    <div className={styles.selectRow}>
                      <div className={styles.selectLabel}>
                        {t("controls.audio_mode.label")}
                      </div>
                      <div className={styles.volumeRow}>
                        <Dropdown
                          className={styles.dropdown}
                          value={t("controls.presets." + settings.preset)}
                          selectedOptions={[settings.preset]}
                          onOptionSelect={(_, data) =>
                            updateSetting("preset", data.optionValue as AudioPreset)
                          }
                        >
                          {presets.map((p) => (
                            <Option key={p} value={p} text={t("controls.presets." + p)}>
                              {t("controls.presets." + p)}
                            </Option>
                          ))}
                        </Dropdown>
                      </div>
                    </div>
                    <div className={styles.selectRow}>
                      <div className={styles.selectLabel}>
                        {t("controls.voice.label")}
                      </div>
                      <div className={styles.volumeRow}>
                        <Dropdown
                          className={styles.dropdown}
                          value={announcementVoiceLabels[settings.announcementVoice]}
                          selectedOptions={[settings.announcementVoice]}
                          onOptionSelect={(_, data) =>
                            updateSetting("announcementVoice", data.optionValue as AnnouncementVoice)
                          }
                        >
                          <Option value="bohdan_hdal" text="Богдан Хдаль">Богдан Хдаль</Option>
                          <Option value="sonia_sotnyk" text="Соня Сотник">Соня Сотник</Option>
                          <Option value="dania_khomutovskyi" text="Даня Хомутовський">
                            Даня Хомутовський
                          </Option>
                          <Option value="air_alert" text="Повітряна тривога">Повітряна тривога</Option>
                        </Dropdown>
                      </div>
                    </div>
                    <div className={styles.selectRow}>
                      <div className={styles.selectLabel}>
                        {t("controls.anthem_voice.label")}
                      </div>
                      <div className={styles.volumeRow}>
                        <Dropdown
                          className={styles.dropdown}
                          value={t("controls.anthem_voice." + settings.anthemVoice)}
                          selectedOptions={[settings.anthemVoice]}
                          onOptionSelect={(_, data) =>
                            updateSetting("anthemVoice", data.optionValue as AnthemVoice)
                          }
                        >
                          <Option value="default" text={t("controls.anthem_voice.default")}>{t("controls.anthem_voice.default")}</Option>
                          <Option value="mykhailo_khoma" text={t("controls.anthem_voice.mykhailo_khoma")}>{t("controls.anthem_voice.mykhailo_khoma")}</Option>
                          <Option value="oleksandr_ponomarov" text={t("controls.anthem_voice.oleksandr_ponomarov")}>{t("controls.anthem_voice.oleksandr_ponomarov")}</Option>
                        </Dropdown>
                      </div>
                    </div>
                    <div>
                      <div className={styles.selectLabel}>
                        {t("controls.volume.label")}
                      </div>
                      <div className={styles.volumeRow}>
                        <Slider
                          className={styles.volumeSlider}
                          min={0}
                          max={100}
                          value={volumeValue}
                          onChange={(_, data) => {
                            setVolumeValue(data.value);
                            updateSetting("volume", data.value);
                          }}
                        />
                        <Text size={200} className={styles.volumeValue}>
                          {volumeValue}%
                        </Text>
                      </div>
                    </div>
                  </Card>

                  <Card className={styles.card}>
                    <Text size={100} weight="semibold" block>
                      СИСТЕМА
                    </Text>
                    <SwitchRow
                      id="pauseToggle"
                      label={t("controls.pause.label")}
                      desc={t("controls.pause.description")}
                      checked={settings.pauseOtherPlayers}
                      onChange={(v) => updateSetting("pauseOtherPlayers", v)}
                    />
                    <Divider />
                    <SwitchRow
                      id="resumeToggle"
                      label={t("controls.resume.label")}
                      desc={t("controls.resume.description")}
                      checked={settings.resumeAfterCeremony}
                      onChange={(v) => updateSetting("resumeAfterCeremony", v)}
                    />
                    <Divider />
                    <SwitchRow
                      id="volumePriorityToggle"
                      label={t("controls.volume_priority.label")}
                      desc={t("controls.volume_priority.description")}
                      checked={settings.volumePriority}
                      onChange={(v) => updateSetting("volumePriority", v)}
                    />
                    <Divider />
                    <SwitchRow
                      id="autoUnmuteToggle"
                      label={t("controls.auto_unmute.label")}
                      desc={t("controls.auto_unmute.description")}
                      checked={settings.autoUnmute}
                      onChange={(v) => updateSetting("autoUnmute", v)}
                    />
                  </Card>

                  <Card className={styles.card}>
                    <div className={styles.infoRow}>
                      <div>
                        {t("status.last_ceremony")}:{" "}
                        <strong>{status.lastActivation ?? "—"}</strong>
                      </div>
                      <div className={styles.ntpRow}>
                        {t("status.ntp_sync")}:{" "}
                        <strong>{status.lastNtpSync ?? "—"}</strong>
                        <Button
                          size="small"
                          appearance="transparent"
                          icon={<ArrowSync20Regular />}
                          disabled={syncing}
                          onClick={handleSyncNtp}
                        >
                          {t("buttons.sync")}
                        </Button>
                      </div>
                    </div>
                  </Card>
                </>
              ) : selectedNav === "about" ? (
                <div className={styles.aboutContent}>
                  <img src="/logo.png" className={styles.aboutLogo} />
                  <div className={styles.aboutTitle}>{t("app.title")}</div>
                  <div className={styles.aboutVersion}>
                    {t("about.version")} {version}
                  </div>
                  <div className={styles.aboutDesc}>
                    {t("about.description")}
                  </div>
                  <div className={styles.aboutDesc}>
                    {t("about.acknowledgments")}
                  </div>
                  <div className={styles.aboutLinks}>
                    <Link
                      onClick={() =>
                        open("https://bohdan.com.ua/memoryminute")
                      }
                    >
                      bohdan.com.ua/memoryminute
                    </Link>
                    <Link
                      onClick={() =>
                        open(
                          "https://github.com/ChernegaSergiy/minute-of-silence"
                        )
                      }
                    >
                      github.com/ChernegaSergiy/minute-of-silence
                    </Link>
                  </div>
                  <div className={styles.aboutLicense}>
                    {t("about.license")}
                    <br />
                    {t("about.glory")}
                  </div>
                </div>
              ) : (
                <div className={styles.changelogContent}>
                  {changelogVersions.map((v) => (
                    <Card key={v.version} className={styles.card}>
                      <CardHeader
                        header={
                          <Text size={200} weight="semibold">
                            v{v.version} — {v.date}
                          </Text>
                        }
                      />
                      {v.categories.map((cat) => (
                        <div key={cat.name} className={styles.changelogCategory}>
                          <Text weight="semibold" size={100} className={styles.changelogCatHeader}>
                            {cat.name}
                          </Text>
                          <ul className={styles.changelogList}>
                            {cat.items.map((item, i) => (
                              <li key={i} className={styles.changelogItem}>
                                <Markdown
                                  components={{
                                    p: ({ children }) => <Text size={200}>{children}</Text>,
                                    code: ({ children }) => <code className={styles.changelogCode}>{children}</code>,
                                    a: ({ href, children }) => (
                                      <Link onClick={() => href && open(href)}>{children}</Link>
                                    ),
                                  }}
                                >
                                  {item}
                                </Markdown>
                              </li>
                            ))}
                          </ul>
                        </div>
                      ))}
                    </Card>
                  ))}
                </div>
              )}
            </div>
          </div>
        </div>

        {selectedNav === "settings" && (
          <div className={styles.buttonBar}>
            <Button
              icon={<Play20Regular />}
              appearance="secondary"
              onClick={triggerCeremonyNow}
            >
              {t("buttons.test")}
            </Button>
            <Button
              icon={<Save20Regular />}
              appearance="primary"
              disabled={!isDirty}
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
