import { useEffect, useMemo, useState } from "react";
import {
  getState,
  onSessionState,
  onSessionTick,
  startSession,
  stopSession,
  type SessionStateDto,
} from "./lib/tauri";

function formatMMSS(sec: number) {
  const minutes = Math.floor(sec / 60);
  const seconds = sec % 60;
  return `${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}`;
}

export default function App() {
  const [state, setState] = useState<SessionStateDto>({
    status: "idle",
    sessionId: null,
    plannedDurationSec: 2700,
    startTsMs: null,
    remainingSec: 0,
  });

  useEffect(() => {
    let unlistenState: (() => void) | undefined;
    let unlistenTick: (() => void) | undefined;

    (async () => {
      const initial = await getState();
      setState(initial);

      unlistenState = await onSessionState((nextState) => {
        setState(nextState);
      });

      unlistenTick = await onSessionTick(({ sessionId, remainingSec }) => {
        setState((prev) =>
          prev.sessionId === sessionId ? { ...prev, remainingSec } : prev,
        );
      });
    })();

    return () => {
      unlistenState?.();
      unlistenTick?.();
    };
  }, []);

  const isRunning = state.status === "running";
  const timeLeft = useMemo(
    () => (isRunning ? formatMMSS(state.remainingSec) : null),
    [isRunning, state.remainingSec],
  );

  const handleStart = async () => {
    try {
      const next = await startSession();
      setState(next);
    } catch (error) {
      console.error("Failed to start session", error);
    }
  };

  const handleStop = async () => {
    try {
      const next = await stopSession();
      setState(next);
    } catch (error) {
      console.error("Failed to stop session", error);
    }
  };

  return (
    <div style={{ padding: 16, fontFamily: "system-ui, -apple-system, sans-serif" }}>
      <h2 style={{ margin: 0 }}>Waypace</h2>
      <p style={{ marginTop: 8 }}>
        Status: <b>{isRunning ? "Running" : "Idle"}</b>
      </p>

      {isRunning && (
        <p style={{ marginTop: 6 }}>
          Time left: <b>{timeLeft}</b>
        </p>
      )}

      <div style={{ display: "flex", gap: 8, marginTop: 12 }}>
        <button
          onClick={handleStart}
          disabled={isRunning}
          style={{
            padding: "8px 12px",
            cursor: isRunning ? "not-allowed" : "pointer",
            flex: 1,
          }}
        >
          Start 45m
        </button>
        <button
          onClick={handleStop}
          disabled={!isRunning}
          style={{
            padding: "8px 12px",
            cursor: !isRunning ? "not-allowed" : "pointer",
            flex: 1,
          }}
        >
          Stop
        </button>
      </div>

      <div style={{ marginTop: 14, fontSize: 12, opacity: 0.7 }}>
        P1.1 — Session + Timer (no DB yet)
      </div>
    </div>
  );
}
