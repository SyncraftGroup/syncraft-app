import { writable } from "svelte/store";
import { isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

const appWindow = isTauri() ? getCurrentWindow() : null;

/**
 * True while the window fills the screen (maximized or fullscreen).
 * The shell drops its rounded corners in that state so it sits flush
 * with the screen edges.
 */
export const maximized = writable(false);

export async function refreshWindowState() {
  if (!appWindow) return;
  const [isMaximized, isFullscreen] = await Promise.all([
    appWindow.isMaximized(),
    appWindow.isFullscreen(),
  ]);
  maximized.set(isMaximized || isFullscreen);
}

/** Subscribes to native window state changes and returns the unlisten fn. */
export async function trackWindowState(): Promise<() => void> {
  if (!appWindow) return () => {};
  await refreshWindowState();
  return appWindow.onResized(() => void refreshWindowState());
}
