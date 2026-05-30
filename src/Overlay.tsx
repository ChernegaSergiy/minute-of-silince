import { useEffect, useRef } from "react";
import UPNG from "upng-js";
import { t } from "./i18n";

interface OverlayProps {
  show: boolean;
  durationSeconds?: number;
}

const candleUrl = "/img/candle_circle.png";
const ringUrl   = "/img/progress_ring.png";

const RING_SIZE   = 260;
const CANDLE_SIZE = RING_SIZE;

const containerStyle: React.CSSProperties = {
  display: "flex",
  position: "fixed",
  top: 0,
  left: 0,
  right: 0,
  bottom: 0,
  backgroundColor: "rgba(0, 0, 0, 0.9)",
  zIndex: 9999,
  justifyContent: "center",
  alignItems: "center",
  flexDirection: "column",
};

const innerStyle: React.CSSProperties = {
  textAlign: "center",
  display: "flex",
  flexDirection: "column",
  alignItems: "center",
  gap: "32px",
};

const mediaWrapperStyle: React.CSSProperties = {
  position: "relative",
  width: `${RING_SIZE}px`,
  height: `${RING_SIZE}px`,
};

const canvasStyle: React.CSSProperties = {
  position: "absolute",
  inset: 0,
  width: "100%",
  height: "100%",
  zIndex: 1,
};

const titleStyle: React.CSSProperties = {
  color: "white",
  fontSize: 24,
  fontWeight: 600,
  textTransform: "uppercase",
  letterSpacing: "0.2em",
};

const subStyle: React.CSSProperties = {
  color: "#888",
  fontSize: 14,
  marginTop: 12,
  textTransform: "uppercase",
  letterSpacing: "0.4em",
};

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

export default function Overlay({ show, durationSeconds = 60 }: OverlayProps) {
  const ringCanvasRef = useRef<HTMLCanvasElement>(null);

  useApngPlayer(ringCanvasRef, ringUrl, durationSeconds, show);

  if (!show) return null;

  return (
    <div style={containerStyle}>
      <div style={innerStyle}>
        <div style={mediaWrapperStyle}>
          <img
            src={candleUrl}
            alt=""
            aria-hidden="true"
            style={{ position: "absolute", inset: 0, width: "100%", height: "100%", objectFit: "contain", zIndex: 0 }}
            width={CANDLE_SIZE}
            height={CANDLE_SIZE}
          />
          <canvas
            ref={ringCanvasRef}
            style={{ ...canvasStyle, zIndex: 1 }}
            aria-hidden="true"
          />
        </div>
        <div style={titleStyle}>{t("overlay.title")}</div>
        <div style={subStyle}>{t("overlay.subtitle")}</div>
      </div>
    </div>
  );
}
