import { t } from "./i18n";

interface OverlayProps {
  show: boolean;
}

const containerStyle: React.CSSProperties = {
  display: "flex",
  position: "fixed",
  top: 0,
  left: 0,
  right: 0,
  bottom: 0,
  backgroundColor: "#000000",
  zIndex: 9999,
  justifyContent: "center",
  alignItems: "center",
  flexDirection: "column",
  userSelect: "none",
  cursor: "none",
};

const innerStyle: React.CSSProperties = {
  textAlign: "center",
  display: "flex",
  flexDirection: "column",
  alignItems: "center",
  gap: "32px",
};

const crossStyle: React.CSSProperties = {
  width: "80px",
  height: "80px",
  backgroundColor: "#cd5c5c",
  clipPath: `polygon(
    35% 0%, 65% 0%, 65% 35%, 100% 35%, 100% 65%,
    65% 65%, 65% 100%, 35% 100%, 35% 65%, 0% 65%, 0% 35%, 35% 35%
  )`,
  animation: "pulse 3s ease-in-out infinite",
};

const titleStyle: React.CSSProperties = {
  color: "white",
  fontSize: 32,
  fontWeight: "bold",
  letterSpacing: "0.25em",
  lineHeight: 1.2,
  textTransform: "uppercase",
};

const subStyle: React.CSSProperties = {
  color: "#888",
  fontSize: 14,
  letterSpacing: "0.5em",
  textTransform: "uppercase",
};

export default function Overlay({ show }: OverlayProps) {
  if (!show) return null;

  return (
    <div style={containerStyle}>
      <style>{`@keyframes pulse { 0% { transform: scale(1); opacity: 1; } 50% { transform: scale(1.02); opacity: 0.8; } 100% { transform: scale(1); opacity: 1; } }`}</style>
      <div style={innerStyle}>
        <div style={crossStyle} />
        <div style={titleStyle}>{t("overlay.title")}</div>
        <div style={subStyle}>{t("overlay.subtitle")}</div>
      </div>
    </div>
  );
}
