import { useEffect, useRef } from "react";
import { t } from "./i18n";
import parseAPNG from "apng-js";
import Player from "apng-js/types/library/player";

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

function useApngPlayer(
  canvasRef: React.RefObject<HTMLCanvasElement | null>,
  src: string,
  durationSeconds: number,
  active: boolean,
) {
  useEffect(() => {
    if (!active) return;
    let player: Player | null = null;
    let isCancelled = false;

    const run = async () => {
      try {
        const resp = await fetch(src);
        const buffer = await resp.arrayBuffer();
        
        if (isCancelled) return;

        const apng = parseAPNG(buffer);
        if (apng instanceof Error) {
          throw apng;
        }

        const canvas = canvasRef.current;
        if (!canvas) return;

        const ctx = canvas.getContext("2d")!;
        
        // Setup canvas size based on APNG dimensions
        canvas.width = apng.width;
        canvas.height = apng.height;

        // Get the built-in apng-js Player that handles blending and disposals automatically
        player = await apng.getPlayer(ctx, false);
        if (isCancelled) {
          player.stop();
          return;
        }

        // Adjust speed rate based on ceremony duration
        player.playbackRate = apng.playTime / (durationSeconds * 1000);
        player.play();
      } catch (e) {
        console.error("APNG play failed:", e);
      }
    };

    run();

    return () => {
      isCancelled = true;
      if (player) {
        player.stop();
      }
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
