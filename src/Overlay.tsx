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

const crossStyle: React.CSSProperties = {
  width: "80px",
  height: "80px",
  backgroundColor: "#cd5c5c",
  clipPath: `polygon(
    35% 0%, 65% 0%, 65% 35%, 100% 35%, 100% 65%,
    65% 65%, 65% 100%, 35% 100%, 35% 65%, 0% 65%, 0% 35%, 35% 35%
  )`,
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

export default function Overlay({ show }: OverlayProps) {
  if (!show) return null;

  return (
    <div style={containerStyle}>
      <div style={innerStyle}>
        <div style={crossStyle} />
        <div style={titleStyle}>{t("overlay.title")}</div>
        <div style={subStyle}>{t("overlay.subtitle")}</div>
      </div>
    </div>
  );
}
