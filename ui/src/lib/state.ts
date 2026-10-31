import { get, writable } from "svelte/store";
import { isTauri } from "@tauri-apps/api/core";
import { api, type Snapshot, type ServerSummary } from "./api";

export const snapshot = writable<Snapshot | null>(null);
export const bootError = writable<string | null>(null);
export const serverError = writable<string | null>(null);

export function patchSnapshot(patch: Partial<Snapshot>) {
  snapshot.update((current) => (current ? { ...current, ...patch } : current));
}

export async function loadSnapshot() {
  if (!isTauri()) return;
  try {
    snapshot.set(await api.snapshot());
    bootError.set(null);
  } catch (err) {
    bootError.set(String(err));
  }
}

export async function refreshServer() {
  if (!isTauri()) return;
  const link = get(snapshot)?.link;
  if (!link) {
    serverError.set(null);
    return;
  }
  serverError.set(null);
  try {
    const server = await api.refreshServer();
    snapshot.update((current) => (current ? { ...current, server } : current));
  } catch (err) {
    serverError.set(String(err));
  }
}

export async function connectServer(link: string): Promise<boolean> {
  if (!isTauri()) return false;
  serverError.set(null);
  try {
    const server: ServerSummary = await api.setServerLink(link);
    snapshot.update((current) =>
      current ? { ...current, server, link: server.link } : current,
    );
    return true;
  } catch (err) {
    serverError.set(String(err));
    return false;
  }
}
