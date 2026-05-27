import { useEffect, useRef } from "react";
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

async function decodeApngFrames(src: string): Promise<ImageBitmap[]> {
  const resp = await fetch(src);
  const blob = await resp.blob();
  // @ts-ignore
  const decoder = new ImageDecoder({ data: blob.stream(), type: "image/png" });
  await decoder.tracks.ready;
  // @ts-ignore
  const count = decoder.tracks.selectedTrack?.frameCount ?? 1;
  const frames: ImageBitmap[] = [];
  for (let i = 0; i < count; i++) {
    // @ts-ignore
    const result = await decoder.decode({ frameIndex: i });
    frames.push(await createImageBitmap(result.image));
    result.image.close();
  }
  decoder.close();
  return frames;
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
    const wallStart = performance.now();

    const run = async () => {
      try { frames = await decodeApngFrames(src); }
      catch (e) { console.error("APNG decode failed:", e); return; }

      const canvas = canvasRef.current;
      if (!canvas || frames.length === 0) return;
      const ctx = canvas.getContext("2d")!;
      const { width, height } = frames[0];
      canvas.width  = width;
      canvas.height = height;

      const tick = (now: number) => {
        const progress = Math.min((now - wallStart) / 1000 / durationSeconds, 1);
        const frameCount = frames.length;
        const frameIdx = frameCount > 1
          ? Math.min(Math.floor(progress * frameCount), frameCount - 1)
          : 0;
        ctx.clearRect(0, 0, width, height);
        ctx.drawImage(frames[frameIdx], 0, 0);
        if (progress < 1) rafId = requestAnimationFrame(tick);
      };
      rafId = requestAnimationFrame(tick);
    };

    run();
    return () => {
      cancelAnimationFrame(rafId);
      frames.forEach((bm) => bm.close());
    };
  }, [active, src, durationSeconds, canvasRef]);
}

export default function Overlay({ show, durationSeconds = 60 }: OverlayProps) {
  const ringCanvasRef   = useRef<HTMLCanvasElement>(null);

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
