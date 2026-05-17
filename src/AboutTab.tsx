import { Link, makeStyles, tokens } from "@fluentui/react-components";
import { open } from "@tauri-apps/plugin-shell";
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
  aboutLicense: {
    fontSize: tokens.fontSizeBase100,
    color: tokens.colorNeutralForeground3,
    marginTop: tokens.spacingVerticalXXL,
  },
});

export default function AboutTab({ version }: AboutTabProps) {
  const styles = useStyles();

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
      <div className={styles.aboutLicense}>
        {t("about.license")}
        <br />
        {t("about.glory")}
      </div>
    </div>
  );
}
