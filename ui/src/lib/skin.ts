import { get, writable } from "svelte/store";
import { isTauri } from "@tauri-apps/api/core";
import { api, type SkinData } from "./api";
import { snapshot } from "./state";

export interface SkinState {
  data: SkinData | null;
  loading: boolean;
  error: string | null;
  uuid: string | null;
}

export const skin = writable<SkinState>({
  data: null,
  loading: false,
  error: null,
  uuid: null,
});

let requestedUuid: string | null | undefined = undefined;
let started = false;

function activeUuid(): string | null {
  const state = get(snapshot);
  const account = state?.account;
  if (account && account.kind === "syncraft" && state?.server) {
    return account.uuid;
  }
  return null;
}

/**
 * Fetch the skin (texture + generated avatar) ahead of time so the account
 * page can paint the avatar immediately.
 *
 * Keeps the current avatar visible while refreshing the same account, and
 * ignores results that arrive after the active account changed.
 */
export async function preloadSkin(force = false) {
  const uuid = activeUuid();

  if (!force && uuid === requestedUuid) return;
  requestedUuid = uuid;

  if (!uuid || !isTauri()) {
    skin.set({ data: null, loading: false, error: null, uuid: null });
    return;
  }

  skin.update((current) => ({
    data: current.uuid === uuid ? current.data : null,
    loading: true,
    error: null,
    uuid,
  }));

  try {
    const data = await api.fetchSkin();
    if (requestedUuid !== uuid) return;
    skin.set({ data, loading: false, error: null, uuid });
  } catch (err) {
    if (requestedUuid !== uuid) return;
    skin.set({ data: null, loading: false, error: String(err), uuid });
  }
}

/** Start preloading whenever the connected account changes. */
export function startSkinPreload() {
  if (started) return;
  started = true;
  snapshot.subscribe(() => {
    void preloadSkin();
  });
}
