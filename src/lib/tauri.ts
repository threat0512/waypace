import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export type SessionStateDto = {
  status: "idle" | "running";
  sessionId: string | null;
  plannedDurationSec: number;
  startTsMs: number | null;
  remainingSec: number;
};

export async function getState(): Promise<SessionStateDto> {
  return invoke("get_state");
}

export async function startSession(
  plannedDurationSec?: number,
): Promise<SessionStateDto> {
  return invoke("start_session", { plannedDurationSec });
}

export async function stopSession(): Promise<SessionStateDto> {
  return invoke("stop_session");
}

export function onSessionState(cb: (state: SessionStateDto) => void) {
  return listen<SessionStateDto>("waypace://session_state", (event) => {
    cb(event.payload);
  });
}

export function onSessionTick(
  cb: (payload: { sessionId: string; remainingSec: number }) => void,
) {
  return listen("waypace://session_tick", (event) => {
    cb(event.payload as { sessionId: string; remainingSec: number });
  });
}
