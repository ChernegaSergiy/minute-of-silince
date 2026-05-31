import { useEffect, useRef, useState, useMemo } from "react";
import {
  makeStyles,
  shorthands,
  tokens,
  Title1,
  Subtitle1,
  FluentProvider,
  webDarkTheme,
  mergeClasses,
} from "@fluentui/react-components";
import UPNG from "upng-js";
import { t } from "../utils/i18n";
import type { PersonalDate } from "../types";

interface OverlayProps {
  show: boolean;
  durationSeconds?: number;
  personalDates?: PersonalDate[];
}

const candleUrl = "/img/candle_circle.png";
const ringUrl   = "/img/progress_ring.png";

const RING_SIZE   = 260;
const CANDLE_SIZE = RING_SIZE;

const useStyles = makeStyles({
  container: {
    display: "flex",
    position: "fixed",
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    backgroundColor: "rgb(8, 8, 8)", // Stable deep solid black base
    zIndex: 9999,
    justifyContent: "center",
    alignItems: "center",
    flexDirection: "column",
    overflow: "hidden",
    userSelect: "none",
    opacity: 0,
    pointerEvents: "none",
    transition: "opacity 1200ms cubic-bezier(0.25, 1, 0.5, 1)",
    fontFamily: tokens.fontFamilyBase,
  },
  containerVisible: {
    opacity: 1,
    pointerEvents: "auto",
  },
  inner: {
    textAlign: "center",
    display: "flex",
    flexDirection: "column",
    alignItems: "center",
    ...shorthands.gap("48px"), // Balanced Fluent UI spacing
    zIndex: 2,
    transform: "scale(0.96)",
    opacity: 0,
    transition: "transform 1400ms cubic-bezier(0.16, 1, 0.3, 1), opacity 1200ms ease-in-out",
  },
  innerVisible: {
    transform: "scale(1)",
    opacity: 1,
  },
  mediaWrapper: {
    position: "relative",
    width: `${RING_SIZE}px`,
    height: `${RING_SIZE}px`,
  },
  canvas: {
    position: "absolute",
    inset: 0,
    width: "100%",
    height: "100%",
    zIndex: 1,
  },
  candle: {
    position: "absolute",
    inset: 0,
    width: "100%",
    height: "100%",
    objectFit: "contain",
    zIndex: 0,
  },
  title: {
    color: tokens.colorNeutralForeground1,
    textTransform: "uppercase",
    letterSpacing: "0.3em",
    fontWeight: tokens.fontWeightSemibold,
    fontSize: "24px",
    margin: 0,
  },
  subtitle: {
    color: tokens.colorNeutralForeground4,
    textTransform: "uppercase",
    letterSpacing: "0.5em",
    fontSize: "13px",
    margin: 0,
  },
  subtitleContainer: {
    display: "flex",
    flexDirection: "column",
    alignItems: "center",
    ...shorthands.gap("8px"),
  },
  personalName: {
    color: tokens.colorNeutralForeground1,
    fontSize: "13px",
    fontWeight: tokens.fontWeightSemibold,
    letterSpacing: "0.4em",
    textTransform: "uppercase",
    textAlign: "center",
    maxWidth: "600px",
    lineHeight: "1.5",
    opacity: 0,
    transform: "translateY(4px)",
    transition: "opacity 500ms ease-in-out, transform 500ms cubic-bezier(0.25, 1, 0.5, 1)",
  },
  personalNameVisible: {
    opacity: 1,
    transform: "translateY(0)",
  }
});

async function loadApngFrames(src: string): Promise<{ frames: ImageBitmap[]; width: number; height: number }> {
  const resp = await fetch(src);
  const buf = await resp.arrayBuffer();
  const img = UPNG.decode(buf);
  const rgbaFrames = UPNG.toRGBA8(img);

  const bitmaps: ImageBitmap[] = [];
  for (const rgbaBuf of rgbaFrames) {
    const rawData = new Uint8ClampedArray(rgbaBuf);
    const imageData = new ImageData(rawData, img.width, img.height);
    const bitmap = await createImageBitmap(imageData);
    bitmaps.push(bitmap);
  }

  return {
    frames: bitmaps,
    width: img.width,
    height: img.height,
  };
}

function useApngPlayer(
  canvasRef: React.RefObject<HTMLCanvasElement | null>,
  src: string,
  durationSeconds: number,
  active: boolean,
) {
  useEffect(() => {
    if (!active) return;
    let rafId: number;
    let frames: ImageBitmap[] = [];
    let startTime: number | null = null;
    let isCancelled = false;

    const run = async () => {
      try {
        const data = await loadApngFrames(src);
        if (isCancelled) {
          data.frames.forEach((bm) => bm.close());
          return;
        }
        frames = data.frames;

        const canvas = canvasRef.current;
        if (!canvas || frames.length === 0) {
          frames.forEach((bm) => bm.close());
          return;
        }

        // Handle high DPI screens
        const dpr = window.devicePixelRatio || 1;
        canvas.width = RING_SIZE * dpr;
        canvas.height = RING_SIZE * dpr;

        const ctx = canvas.getContext("2d")!;
        ctx.scale(dpr, dpr);

        const tick = (now: number) => {
          if (!startTime) startTime = now;
          const elapsed = (now - startTime) / 1000;
          const progress = Math.min(elapsed / durationSeconds, 1);
          const frameIdx = Math.min(Math.floor(progress * frames.length), frames.length - 1);

          ctx.clearRect(0, 0, RING_SIZE, RING_SIZE);
          if (frames[frameIdx]) {
            ctx.drawImage(frames[frameIdx], 0, 0, RING_SIZE, RING_SIZE);
          }

          if (progress < 1) {
            rafId = requestAnimationFrame(tick);
          }
        };
        rafId = requestAnimationFrame(tick);
      } catch (e) {
        console.error("APNG decode failed:", e);
      }
    };

    run();

    return () => {
      isCancelled = true;
      cancelAnimationFrame(rafId);
      frames.forEach((bm) => bm.close());
    };
  }, [active, src, durationSeconds, canvasRef]);
}

export default function Overlay({ show, durationSeconds = 60, personalDates = [] }: OverlayProps) {
  const styles = useStyles();
  const ringCanvasRef = useRef<HTMLCanvasElement>(null);

  // Manage mounting delay for smooth transitions
  const [shouldRender, setShouldRender] = useState(show);
  const [visible, setVisible] = useState(false);

  useEffect(() => {
    if (show) {
      setShouldRender(true);
      const t = setTimeout(() => setVisible(true), 50);
      return () => clearTimeout(t);
    } else {
      setVisible(false);
      const t = setTimeout(() => setShouldRender(false), 1200); // Wait for transition to finish
      return () => clearTimeout(t);
    }
  }, [show]);

  useApngPlayer(ringCanvasRef, ringUrl, durationSeconds, shouldRender);

  // Get active personal dates matching today
  const activeDates = useMemo(() => {
    const today = new Date();
    const currentMonth = today.getMonth() + 1;
    const currentDay = today.getDate();
    const currentYear = today.getFullYear();
    const isLeapYear = (y: number) => (y % 4 === 0 && y % 100 !== 0) || y % 400 === 0;

    let active = personalDates.filter(
      (d) => d.month === currentMonth && d.day === currentDay
    );

    if (currentMonth === 2 && currentDay === 28 && !isLeapYear(currentYear)) {
      const feb29Events = personalDates.filter((d) => d.month === 2 && d.day === 29);
      active = [...active, ...feb29Events];
    }
    return active;
  }, [personalDates]);

  const hasActiveDates = activeDates.length > 0;

  // Name carousel/slider state
  const [currentNameIndex, setCurrentNameIndex] = useState(0);
  const [nameFadeState, setNameFadeState] = useState(true);

  // Reset index when active dates or overlay state changes
  useEffect(() => {
    setCurrentNameIndex(0);
    setNameFadeState(true);
  }, [activeDates, show]);

  // Rotator effect for multiple names
  useEffect(() => {
    if (activeDates.length <= 1 || !show) return;

    const interval = setInterval(() => {
      setNameFadeState(false); // Start fade-out animation

      setTimeout(() => {
        setCurrentNameIndex((prev) => (prev + 1) % activeDates.length);
        setNameFadeState(true); // Start fade-in animation
      }, 500);
    }, 4000); // 4 seconds total interval for each slide

    return () => clearInterval(interval);
  }, [activeDates, show]);

  const currentCommemorationName = activeDates[currentNameIndex]?.label || "";

  if (!shouldRender) return null;

  return (
    <FluentProvider theme={webDarkTheme}>
      <div className={mergeClasses(styles.container, visible && styles.containerVisible)}>
        <div className={mergeClasses(styles.inner, visible && styles.innerVisible)}>
          <Title1 className={styles.title}>{t("overlay.title")}</Title1>
          
          <div className={styles.mediaWrapper}>
            <img
              src={candleUrl}
              alt=""
              aria-hidden="true"
              className={styles.candle}
              width={CANDLE_SIZE}
              height={CANDLE_SIZE}
            />
            <canvas
              ref={ringCanvasRef}
              className={styles.canvas}
              aria-hidden="true"
            />
          </div>
          
          <div className={styles.subtitleContainer}>
            <Subtitle1 className={styles.subtitle}>
              {hasActiveDates ? t("overlay.personal_subtitle") : t("overlay.subtitle")}
            </Subtitle1>
            {hasActiveDates && currentCommemorationName && (
              <div
                className={mergeClasses(
                  styles.personalName,
                  nameFadeState && styles.personalNameVisible
                )}
              >
                {currentCommemorationName}
              </div>
            )}
          </div>
        </div>
      </div>
    </FluentProvider>
  );
}
