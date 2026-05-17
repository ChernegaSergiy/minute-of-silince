import {
  Button,
  Card,
  Divider,
  Dropdown,
  Option,
  Slider,
  Switch,
  Text,
  makeStyles,
  tokens,
} from "@fluentui/react-components";
import { ArrowSync20Regular } from "@fluentui/react-icons";
import { skipNext, unskipNext } from "./api";
import type { AnnouncementVoice, AnthemVoice, AudioPreset, Settings, StatusSnapshot } from "./types";
import { t } from "./i18n";

const presets: AudioPreset[] = [
  "voice_metronome",
  "metronome_only",
  "voice_silence_bell",
  "voice_silence",
  "voice_metronome_anthem",
  "voice_metronome_ending",
  "metronome_anthem",
  "bell_silence_bell",
  "bell_metronome_bell",
  "silence",
];

const presetLabel = (preset: AudioPreset) => t(`controls.presets.${preset}`);

const anthemPresets = ["voice_metronome_anthem", "metronome_anthem"];

const announcementVoices: AnnouncementVoice[] = [
  "bohdan_hdal",
  "sonia_sotnyk",
  "dania_khomutovskyi",
  "air_alert",
];

const announcementVoiceLabel = (voice: AnnouncementVoice) => t(`controls.voice.${voice}`);

type UpdateSetting = <K extends keyof Settings>(key: K, value: Settings[K]) => void;

interface SettingsTabProps {
  settings: Settings;
  status: StatusSnapshot;
  volumeValue: number;
  syncing: boolean;
  onUpdateSetting: UpdateSetting;
  onVolumeChange: (value: number) => void;
  onSyncNtp: () => void;
}

const useStyles = makeStyles({
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

export default function SettingsTab({
  settings,
  status,
  volumeValue,
  syncing,
  onUpdateSetting,
  onVolumeChange,
  onSyncNtp,
}: SettingsTabProps) {
  const styles = useStyles();

  return (
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
          onChange={(v) => onUpdateSetting("ceremonyEnabled", v)}
        />
        <Divider />
        <SwitchRow
          id="autostartToggle"
          label={t("controls.autostart.label")}
          desc={t("controls.autostart.description")}
          checked={settings.autostartEnabled}
          onChange={(v) => onUpdateSetting("autostartEnabled", v)}
        />
        <Divider />
        <SwitchRow
          id="weekdaysToggle"
          label={t("controls.weekdays.label")}
          desc={t("controls.weekdays.description")}
          checked={settings.weekdaysOnly}
          onChange={(v) => onUpdateSetting("weekdaysOnly", v)}
        />
        <Divider />
        <div>
          <div className={styles.selectLabel}>{t("controls.grace.label")}</div>
          <div className={styles.volumeRow}>
            <Slider
              className={styles.volumeSlider}
              min={0}
              max={5}
              value={settings.lateStartGraceMinutes}
              onChange={(_, data) => {
                onUpdateSetting("lateStartGraceMinutes", data.value);
              }}
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
          onChange={(v) => onUpdateSetting("systemTimeOnly", v)}
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
          onChange={(v) => onUpdateSetting("reminderEnabled", v)}
        />
        <Divider />
        {settings.reminderEnabled && (
          <>
            <div className={styles.volumeRow}>
              <Dropdown
                className={styles.dropdown}
                value={
                  settings.reminderMinutesBefore === 0
                    ? t("controls.reminder.immediately")
                    : `${t("controls.reminder.in")} ${settings.reminderMinutesBefore} ${t("controls.grace.unit")}`
                }
                selectedOptions={[String(settings.reminderMinutesBefore)]}
                onOptionSelect={(_, data) =>
                  onUpdateSetting("reminderMinutesBefore", Number(data.optionValue))
                }
              >
                <Option value="0" text={t("controls.reminder.immediately")}>
                  {t("controls.reminder.immediately")}
                </Option>
                {[1, 2, 3, 4, 5, 6, 7, 8, 9, 10].map((n) => (
                  <Option
                    key={n}
                    value={String(n)}
                    text={`${t("controls.reminder.in")} ${n} ${t("controls.grace.unit")}`}
                  >
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
          onChange={(v) => onUpdateSetting("showVisualOverlay", v)}
        />
        {settings.showVisualOverlay && (
          <>
            <Divider />
            <SwitchRow
              id="flagAnimationToggle"
              label={t("controls.flag_animation.label")}
              desc={t("controls.flag_animation.description")}
              checked={settings.showFlagAnimation}
              onChange={(v) => onUpdateSetting("showFlagAnimation", v)}
            />
          </>
        )}
      </Card>

      <Card className={styles.card}>
        <Text size={100} weight="semibold" block>
          {t("controls.audio_mode.label")}
        </Text>
        <div className={styles.selectRow}>
          <div className={styles.selectLabel}>{t("controls.audio_mode.label")}</div>
          <div className={styles.volumeRow}>
            <Dropdown
              className={styles.dropdown}
              value={presetLabel(settings.preset)}
              selectedOptions={[settings.preset]}
              onOptionSelect={(_, data) =>
                onUpdateSetting("preset", data.optionValue as AudioPreset)
              }
            >
              {presets.map((p) => (
                <Option key={p} value={p} text={presetLabel(p)}>
                  {presetLabel(p)}
                </Option>
              ))}
            </Dropdown>
          </div>
        </div>
        <div className={styles.selectRow}>
          <div className={styles.selectLabel}>{t("controls.voice.label")}</div>
          <div className={styles.volumeRow}>
            <Dropdown
              className={styles.dropdown}
              value={announcementVoiceLabel(settings.announcementVoice)}
              selectedOptions={[settings.announcementVoice]}
              onOptionSelect={(_, data) =>
                onUpdateSetting("announcementVoice", data.optionValue as AnnouncementVoice)
              }
            >
              {announcementVoices.map((voice) => (
                <Option key={voice} value={voice} text={announcementVoiceLabel(voice)}>
                  {announcementVoiceLabel(voice)}
                </Option>
              ))}
            </Dropdown>
          </div>
        </div>
        {anthemPresets.includes(settings.preset) && (
          <div className={styles.selectRow}>
            <div className={styles.selectLabel}>{t("controls.anthem_voice.label")}</div>
            <div className={styles.volumeRow}>
              <Dropdown
                className={styles.dropdown}
                value={t("controls.anthem_voice." + settings.anthemVoice)}
                selectedOptions={[settings.anthemVoice]}
                onOptionSelect={(_, data) =>
                  onUpdateSetting("anthemVoice", data.optionValue as AnthemVoice)
                }
              >
                <Option value="default" text={t("controls.anthem_voice.default")}>
                  {t("controls.anthem_voice.default")}
                </Option>
                <Option value="mykhailo_khoma" text={t("controls.anthem_voice.mykhailo_khoma")}>
                  {t("controls.anthem_voice.mykhailo_khoma")}
                </Option>
                <Option value="oleksandr_ponomarov" text={t("controls.anthem_voice.oleksandr_ponomarov")}>
                  {t("controls.anthem_voice.oleksandr_ponomarov")}
                </Option>
              </Dropdown>
            </div>
          </div>
        )}
        <div>
          <div className={styles.selectLabel}>{t("controls.volume.label")}</div>
          <div className={styles.volumeRow}>
            <Slider
              className={styles.volumeSlider}
              min={0}
              max={100}
              value={volumeValue}
              onChange={(_, data) => {
                onVolumeChange(data.value);
                onUpdateSetting("volume", data.value);
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
          onChange={(v) => onUpdateSetting("pauseOtherPlayers", v)}
        />
        <Divider />
        {settings.pauseOtherPlayers && (
          <>
            <SwitchRow
              id="resumeToggle"
              label={t("controls.resume.label")}
              desc={t("controls.resume.description")}
              checked={settings.resumeAfterCeremony}
              onChange={(v) => onUpdateSetting("resumeAfterCeremony", v)}
            />
            <Divider />
          </>
        )}
        <SwitchRow
          id="volumePriorityToggle"
          label={t("controls.volume_priority.label")}
          desc={t("controls.volume_priority.description")}
          checked={settings.volumePriority}
          onChange={(v) => onUpdateSetting("volumePriority", v)}
        />
        <Divider />
        <SwitchRow
          id="autoUnmuteToggle"
          label={t("controls.auto_unmute.label")}
          desc={t("controls.auto_unmute.description")}
          checked={settings.autoUnmute}
          onChange={(v) => onUpdateSetting("autoUnmute", v)}
        />
      </Card>

      <Card className={styles.statusCard}>
        <div className={styles.infoRow}>
          <div>
            {t("status.last_ceremony")}: <strong>{status.lastActivation ?? "—"}</strong>
          </div>
          <div className={styles.ntpRow}>
            {t("status.ntp_sync")}: <strong>{status.lastNtpSync ?? "—"}</strong>
            <Button
              size="small"
              appearance="transparent"
              icon={<ArrowSync20Regular />}
              disabled={syncing}
              onClick={onSyncNtp}
            >
              {t("buttons.sync")}
            </Button>
          </div>
        </div>
      </Card>
    </>
  );
}
