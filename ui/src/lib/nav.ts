import { get, writable } from "svelte/store";

export const PAGES = ["home", "server", "account", "settings"] as const;
export type Page = (typeof PAGES)[number];

function readPage(): Page {
  if (typeof window === "undefined") return "home";
  const value = window.location.hash.replace(/^#\/?/, "");
  return (PAGES as readonly string[]).includes(value) ? (value as Page) : "home";
}

export const page = writable<Page>(readPage());

if (typeof window !== "undefined") {
  window.addEventListener("hashchange", () => page.set(readPage()));
}

export function navigate(next: Page) {
  if (get(page) === next) return;
  // Update the store synchronously so the page swaps on the same tick, then
  // mirror it into the URL without a hashchange round-trip.
  page.set(next);
  if (typeof window !== "undefined" && readPage() !== next) {
    window.history.replaceState(null, "", `#/${next}`);
  }
}
