import { writable } from "svelte/store";

/**
 * M5 hook: the server will eventually provide a sanitized HTML/CSS document
 * here. Keeping this store in M0 means the home layout never has to change
 * when custom homepages land.
 */
export interface HomeDoc {
  html: string;
  css: string;
  hash?: string;
}

export const homeDoc = writable<HomeDoc | null>(null);
