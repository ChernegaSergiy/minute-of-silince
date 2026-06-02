import { useCallback, useEffect, useState } from "react";
import {
  Dialog,
  DialogSurface,
  DialogTitle,
  DialogContent,
  DialogBody,
  DialogActions,
  Button,
  ProgressBar,
  Text,
  makeStyles,
  tokens,
  shorthands,
  Link,
} from "@fluentui/react-components";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-shell";
import Markdown from "react-markdown";
import { t } from "../utils/i18n";

export interface UpdateInfo {
  version: string;
  currentVersion: string;
  body?: string;
}

interface UpdateDialogProps {
  updateInfo: UpdateInfo | null;
  onClose: () => void;
}

const useStyles = makeStyles({
  dialogSurface: {
    maxWidth: "500px",
    width: "90%",
  },
  title: {
    ...shorthands.margin(0, 0, tokens.spacingVerticalS, 0),
  },
  versionInfo: {
    marginBottom: tokens.spacingVerticalM,
    color: tokens.colorNeutralForeground3,
    fontSize: tokens.fontSizeBase200,
  },
  changelogTitle: {
    fontWeight: tokens.fontWeightSemibold,
    marginBottom: tokens.spacingVerticalXS,
    display: "block",
  },
  changelogScroll: {
    maxHeight: "200px",
    overflowY: "auto",
    ...shorthands.border("1px", "solid", tokens.colorNeutralStroke2),
    ...shorthands.padding(tokens.spacingVerticalS, tokens.spacingHorizontalS),
    borderRadius: tokens.borderRadiusMedium,
    backgroundColor: tokens.colorNeutralBackground2,
    marginBottom: tokens.spacingVerticalM,
  },
  changelogCode: {
    fontSize: tokens.fontSizeBase200,
    backgroundColor: tokens.colorNeutralBackground5,
    padding: `0 ${tokens.spacingHorizontalXXS}`,
    borderRadius: tokens.borderRadiusSmall,
  },
  progressContainer: {
    marginTop: tokens.spacingVerticalM,
    marginBottom: tokens.spacingVerticalM,
  },
  progressText: {
    display: "block",
    textAlign: "center",
    marginTop: tokens.spacingVerticalXS,
    color: tokens.colorNeutralForeground2,
    fontSize: tokens.fontSizeBase200,
  },
  errorText: {
    color: tokens.colorPaletteRedBorderActive,
    marginTop: tokens.spacingVerticalS,
    display: "block",
    fontSize: tokens.fontSizeBase200,
  },
});

export default function UpdateDialog({ updateInfo, onClose }: UpdateDialogProps) {
  const styles = useStyles();
  const [updating, setUpdating] = useState(false);
  const [progress, setProgress] = useState<number | null>(null);
  const [statusText, setStatusText] = useState<string>("");
  const [errorMsg, setErrorMsg] = useState<string>("");

  useEffect(() => {
    if (!updateInfo) {
      setUpdating(false);
      setProgress(null);
      setStatusText("");
      setErrorMsg("");
    }
  }, [updateInfo]);

  useEffect(() => {
    let unlistenProgress: (() => void) | null = null;

    const setupListener = async () => {
      unlistenProgress = await listen<{ progress: number; status: string }>(
        "update-progress",
        (event) => {
          const { progress, status } = event.payload;
          setProgress(progress);
          if (status === "downloading") {
            setStatusText(t("update.status_downloading", { progress: Math.round(progress) }));
          } else if (status === "installing") {
            setStatusText(t("update.status_installing"));
          } else if (status === "restarting") {
            setStatusText(t("update.status_restarting"));
          }
        }
      );
    };

    if (updateInfo) {
      setupListener();
    }

    return () => {
      if (unlistenProgress) {
        unlistenProgress();
      }
    };
  }, [updateInfo]);

  const handleInstall = useCallback(async () => {
    setUpdating(true);
    setErrorMsg("");
    setStatusText(t("update.status_downloading", { progress: 0 }));
    setProgress(0);

    try {
      await invoke("install_update");
    } catch (err) {
      setErrorMsg(String(err));
      setUpdating(false);
      setProgress(null);
      setStatusText("");
    }
  }, []);

  if (!updateInfo) return null;

  return (
    <Dialog modalType="alert" open={!!updateInfo} onOpenChange={() => {
      if (!updating) {
        onClose();
      }
    }}>
      <DialogSurface className={styles.dialogSurface}>
        <DialogBody>
          <DialogTitle className={styles.title}>{t("update.title")}</DialogTitle>
          <DialogContent>
            <div className={styles.versionInfo}>
              {t("update.version_info", {
                newVersion: updateInfo.version,
                currentVersion: updateInfo.currentVersion,
              })}
            </div>

            {updateInfo.body && (
              <>
                <Text className={styles.changelogTitle} size={200}>
                  {t("update.changelog")}
                </Text>
                <div className={styles.changelogScroll}>
                  <Markdown
                    components={{
                      p: ({ children }) => <Text size={200} block style={{ marginBottom: "4px" }}>{children}</Text>,
                      code: ({ children }) => <code className={styles.changelogCode}>{children}</code>,
                      a: ({ href, children }) => (
                        <Link onClick={() => href && open(href)}>{children}</Link>
                      ),
                      li: ({ children }) => <li style={{ fontSize: "12px", lineHeight: "1.4" }}>{children}</li>,
                    }}
                  >
                    {updateInfo.body}
                  </Markdown>
                </div>
              </>
            )}

            {progress !== null && (
              <div className={styles.progressContainer}>
                <ProgressBar value={progress / 100} />
                <span className={styles.progressText}>{statusText}</span>
              </div>
            )}

            {errorMsg && (
              <span className={styles.errorText}>
                {t("update.error", { error: errorMsg })}
              </span>
            )}
          </DialogContent>
          <DialogActions>
            <Button
              appearance="primary"
              disabled={updating}
              onClick={handleInstall}
            >
              {t("update.btn_install")}
            </Button>
            <Button
              appearance="secondary"
              disabled={updating}
              onClick={onClose}
            >
              {t("update.btn_later")}
            </Button>
          </DialogActions>
        </DialogBody>
      </DialogSurface>
    </Dialog>
  );
}
