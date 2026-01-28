export default function App() {
  return (
    <div
      style={{
        padding: 16,
        fontFamily: "system-ui, -apple-system, sans-serif",
      }}
    >
      <h2 style={{ margin: 0 }}>Waypace</h2>
      <p style={{ marginTop: 8 }}>
        Status: <b>Idle</b>
      </p>

      <button
        style={{
          marginTop: 12,
          padding: "8px 12px",
          cursor: "not-allowed",
          opacity: 0.6,
        }}
        disabled
      >
        Start Session (P1.1)
      </button>

      <div style={{ marginTop: 18, fontSize: 12, opacity: 0.7 }}>
        Tray app scaffold — P1.0
      </div>
    </div>
  );
}
