import { get, writable } from "svelte/store";
import { isTauri } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { api, type LaunchEvent, type Phase } from "./api";

export interface LaunchState {
  phase: Phase;
  progress: number;
  stage: string;
  filesDone: number;
  filesTotal: number;
  bytesDone: number;
  bytesTotal: number;
  speed: number;
  error: string | null;
}

const initial: LaunchState = {
  phase: "idle",
  progress: 0,
  stage: "",
  filesDone: 0,
  filesTotal: 0,
  bytesDone: 0,
  bytesTotal: 0,
  speed: 0,
  error: null,
};

export const launch = writable<LaunchState>(initial);

let unlisten: UnlistenFn | null = null;
let lastPhase: Phase = "idle";
let lastBytes = 0;
let lastTick = 0;
let speed = 0;

function apply(event: LaunchEvent) {
  const now = performance.now();

  if (event.phase !== lastPhase) {
    lastPhase = event.phase;
    lastBytes = event.bytesDone;
    lastTick = now;
    speed = 0;
  } else if (lastTick > 0 && event.bytesDone >= lastBytes) {
    const seconds = (now - lastTick) / 1000;
    if (seconds > 0) {
      const instant = (event.bytesDone - lastBytes) / seconds;
      speed = speed > 0 ? speed * 0.65 + instant * 0.35 : instant;
    }
    lastBytes = event.bytesDone;
    lastTick = now;
  }

  launch.set({
    phase: event.phase,
    progress: event.progress,
    stage: event.stage,
    filesDone: event.filesDone,
    filesTotal: event.filesTotal,
    bytesDone: event.bytesDone,
    bytesTotal: event.bytesTotal,
    speed,
    error: event.message ?? null,
  });
}

// Downloads can stall without producing events; decay the reported speed then.
if (typeof window !== "undefined") {
  window.setInterval(() => {
    if (lastTick === 0 || performance.now() - lastTick < 1200) return;
    speed = 0;
    launch.update((state) =>
      state.phase === "idle" || state.phase === "error" ? state : { ...state, speed: 0 },
    );
  }, 500);
}

export async function initLaunchEvents() {
  if (!isTauri() || unlisten) return;
  unlisten = await listen<LaunchEvent>("launch-progress", (event) => {
    apply(event.payload);
  });
}

export async function startLaunch() {
  const current = get(launch);
  if (!["idle", "error"].includes(current.phase)) return;

  launch.set({ ...initial, phase: "checking", stage: "正在检查更新" });

  try {
    await api.startLaunch();
  } catch (err) {
    launch.set({ ...initial, phase: "error", stage: "启动失败", error: String(err) });
  }
}

export function resetLaunch() {
  lastPhase = "idle";
  lastBytes = 0;
  lastTick = 0;
  speed = 0;
  launch.set(initial);
}
