import { Button, Link, makeStyles, tokens } from "@fluentui/react-components";
import { ClipboardRegular } from "@fluentui/react-icons";
import { open } from "@tauri-apps/plugin-shell";
import { getLogContents } from "./api";
import { t } from "./i18n";

interface AboutTabProps {
  version: string;
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
  },
  aboutLicense: {
    fontSize: tokens.fontSizeBase100,
    color: tokens.colorNeutralForeground3,
    marginTop: tokens.spacingVerticalXXL,
  },
});

export default function AboutTab({ version }: AboutTabProps) {
  const styles = useStyles();

  const handleCopyLogs = async () => {
    const logs = await getLogContents();
    await navigator.clipboard.writeText(logs);
  };

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
        <Button appearance="subtle" icon={<ClipboardRegular />} onClick={handleCopyLogs}>
          {t("about.copy_logs")}
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
