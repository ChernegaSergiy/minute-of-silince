import { Button, Link, makeStyles, tokens } from "@fluentui/react-components";
import { ArrowSyncRegular, ClipboardCheckmarkRegular, ClipboardRegular } from "@fluentui/react-icons";
import { useCallback, useState } from "react";
import { open } from "@tauri-apps/plugin-shell";
import { getLogContents } from "../utils/api";
import { t } from "../utils/i18n";
import { type UpdateInfo } from "./UpdateDialog";

interface AboutTabProps {
  version: string;
  onCheckForUpdates: () => Promise<UpdateInfo | null>;
  onUpdateFound: (update: UpdateInfo) => void;
}

const useStyles = makeStyles({
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
  aboutTools: {
    marginTop: tokens.spacingVerticalL,
    display: "flex",
    justifyContent: "center",
    gap: tokens.spacingHorizontalM,
  },
  aboutLicense: {
    fontSize: tokens.fontSizeBase100,
    color: tokens.colorNeutralForeground3,
    marginTop: tokens.spacingVerticalXXL,
  },
  spinIcon: {
    animationName: {
      from: { transform: "rotate(0deg)" },
      to: { transform: "rotate(360deg)" },
    },
    animationDuration: "1s",
    animationIterationCount: "infinite",
    animationTimingFunction: "linear",
  },
});

export default function AboutTab({ version, onCheckForUpdates, onUpdateFound }: AboutTabProps) {
  const styles = useStyles();
  const [copyState, setCopyState] = useState<"idle" | "copied" | "error">("idle");
  const [updateCheckState, setUpdateCheckState] = useState<"initial" | "checking" | "up_to_date" | "error">("initial");

  const handleCheckUpdates = useCallback(async () => {
    if (updateCheckState === "checking") return;
    setUpdateCheckState("checking");
    try {
      const [update] = await Promise.all([
        onCheckForUpdates(),
        new Promise((resolve) => setTimeout(resolve, 800)),
      ]);

      if (update) {
        setUpdateCheckState("initial");
        onUpdateFound(update);
      } else {
        setUpdateCheckState("up_to_date");
        window.setTimeout(() => setUpdateCheckState("initial"), 2000);
      }
    } catch {
      setUpdateCheckState("error");
      window.setTimeout(() => setUpdateCheckState("initial"), 2000);
    }
  }, [updateCheckState, onCheckForUpdates, onUpdateFound]);

  const handleCopyLogs = useCallback(async () => {
    if (copyState !== "idle") return;
    try {
      const logs = await getLogContents();
      await navigator.clipboard.writeText(logs);
      setCopyState("copied");
    } catch {
      setCopyState("error");
    }

    window.setTimeout(() => setCopyState("idle"), 2000);
  }, [copyState]);

  return (
    <div className={styles.aboutContent}>
      <img src="/logo.png" className={styles.aboutLogo} />
      <div className={styles.aboutTitle}>{t("app.title")}</div>
      <div className={styles.aboutVersion}>
        {t("about.version")} {version}
      </div>
      <div className={styles.aboutDesc}>{t("about.description")}</div>
      <div className={styles.aboutDesc}>{t("about.acknowledgments")}</div>
      <div className={styles.aboutLinks}>
        <Link onClick={() => open("https://bohdan.com.ua/memoryminute")}>
          bohdan.com.ua/memoryminute
        </Link>
        <Link
          onClick={() =>
            open("https://github.com/ChernegaSergiy/minute-of-silence")
          }
        >
          github.com/ChernegaSergiy/minute-of-silence
        </Link>
      </div>
      <div className={styles.aboutTools}>
        <Button
          key={updateCheckState}
          appearance="subtle"
          icon={
            <ArrowSyncRegular
              className={updateCheckState === "checking" ? styles.spinIcon : undefined}
            />
          }
          onClick={handleCheckUpdates}
          disabled={updateCheckState === "checking"}
        >
          {updateCheckState === "checking"
            ? t("about.check_updates_checking")
            : updateCheckState === "up_to_date"
              ? t("about.check_updates_up_to_date")
              : updateCheckState === "error"
                ? t("about.check_updates_error")
                : t("about.check_updates")}
        </Button>
        <Button
          key={copyState}
          appearance="subtle"
          icon={copyState === "copied" ? <ClipboardCheckmarkRegular /> : <ClipboardRegular />}
          onClick={handleCopyLogs}
        >
          {copyState === "copied"
            ? t("about.copy_logs_copied")
            : copyState === "error"
              ? t("about.copy_logs_error")
              : t("about.copy_logs")}
        </Button>
      </div>
      <div className={styles.aboutLicense}>
        {t("about.license")}
        <br />
        {t("about.glory")}
      </div>
    </div>
  );
}
