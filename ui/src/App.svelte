<script lang="ts">
  import { onMount } from "svelte";
  import { get } from "svelte/store";
  import { isTauri } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import TopBar from "./components/TopBar.svelte";
  import ActionBar from "./components/ActionBar.svelte";
  import HomePage from "./pages/HomePage.svelte";
  import ServerPage from "./pages/ServerPage.svelte";
  import AccountPage from "./pages/AccountPage.svelte";
  import SettingsPage from "./pages/SettingsPage.svelte";
  import { page, navigate, type Page } from "./lib/nav";
  import { launch, startLaunch, initLaunchEvents } from "./lib/launch";
  import { loadSnapshot, refreshServer, snapshot } from "./lib/state";
  import { startSkinPreload } from "./lib/skin";
  import { maximized, trackWindowState } from "./lib/window";

  const titles: Record<Page, string> = {
    home: "Syncraft",
    server: "服务器 · Syncraft",
    account: "账号 · Syncraft",
    settings: "设置 · Syncraft",
  };

  const appWindow = isTauri() ? getCurrentWindow() : null;
  type ResizeDirection =
    | "North"
    | "NorthEast"
    | "East"
    | "SouthEast"
    | "South"
    | "SouthWest"
    | "West"
    | "NorthWest";
  const resizeDirections: ResizeDirection[] = [
    "North",
    "NorthEast",
    "East",
    "SouthEast",
    "South",
    "SouthWest",
    "West",
    "NorthWest",
  ];

  function startResize(event: PointerEvent, direction: ResizeDirection) {
    event.preventDefault();
    if (appWindow) void appWindow.startResizeDragging(direction);
  }

  $: if (typeof document !== "undefined") {
    document.title = titles[$page];
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key !== "Enter" || event.defaultPrevented) return;
    if ($page !== "home" || $launch.phase !== "idle") return;

    const target = event.target;
    if (target instanceof HTMLElement && target.closest("button, a, input, select, textarea")) {
      return;
    }

    event.preventDefault();
    const state = get(snapshot);
    if (!state?.server) {
      navigate("server");
      return;
    }
    if (!state.account) {
      navigate("account");
      return;
    }
    void startLaunch();
  }

  onMount(() => {
    let dispose: (() => void) | undefined;
    let cancelled = false;

    if (appWindow) {
      void trackWindowState().then((stop) => {
        if (cancelled) stop();
        else dispose = stop;
      });
    }

    if (isTauri()) {
      startSkinPreload();
      void initLaunchEvents();

      // Parse the lazily loaded 3D engine while the app is idle so opening the
      // account page only pays for the WebGL context, not the module compile.
      const warm3d = () => void import("skinview3d").catch(() => {});
      const withIdle = window as unknown as {
        requestIdleCallback?: (callback: () => void, options?: { timeout: number }) => number;
      };
      if (typeof withIdle.requestIdleCallback === "function") {
        withIdle.requestIdleCallback(warm3d, { timeout: 5000 });
      } else {
        window.setTimeout(warm3d, 2500);
      }
      void loadSnapshot().then(() => {
        const state = get(snapshot);
        if (!state) return;
        if (state.link) void refreshServer();
        if (state.autoLaunch && state.server && state.account && get(launch).phase === "idle") {
          void startLaunch();
        }
      });
    }

    return () => {
      cancelled = true;
      dispose?.();
    };
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="shell" class:flush={$maximized}>
  <TopBar />

  <main class="main">
    {#key $page}
      <div class="page-view">
        {#if $page === "home"}
          <HomePage />
        {:else if $page === "server"}
          <ServerPage />
        {:else if $page === "account"}
          <AccountPage />
        {:else}
          <SettingsPage />
        {/if}
      </div>
    {/key}
  </main>

  <ActionBar />

  {#if appWindow}
    <div class="resize-layer">
      {#each resizeDirections as direction}
        <button
          type="button"
          class="resize-handle {direction.toLowerCase()}"
          tabindex="-1"
          aria-label="调整窗口大小"
          onpointerdown={(event) => startResize(event, direction)}
        ></button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .shell {
    position: relative;
    display: flex;
    height: 100%;
    flex-direction: column;
    overflow: hidden;
    border: var(--window-side) solid var(--ink);
    border-top-width: var(--window-edge);
    border-radius: var(--window-radius);
    /* The paper stops at the inner edge of the frame, so the rounded outer
       edge only ever blends ink with the desktop — no white fringe. */
    background: var(--paper);
    background-clip: padding-box;
  }

  .shell.flush {
    --window-edge: 0px;
    --window-side: 0px;
    border-color: transparent;
    border-radius: 0;
  }

  .main {
    position: relative;
    min-height: 0;
    flex: 1;
  }

  .page-view {
    height: 100%;
    animation: page-in 280ms cubic-bezier(0.22, 0.61, 0.36, 1) both;
  }

  @keyframes page-in {
    from {
      opacity: 0;
      transform: translateY(8px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .page-view {
      animation: none;
    }
  }

  .resize-layer {
    position: fixed;
    inset: 0;
    z-index: 1000;
    pointer-events: none;
  }

  .resize-handle {
    position: absolute;
    padding: 0;
    border: 0;
    background: transparent;
    pointer-events: auto;
  }

  .resize-handle.north {
    top: 0;
    right: 8px;
    left: 8px;
    height: 4px;
    cursor: ns-resize;
  }

  .resize-handle.south {
    right: 8px;
    bottom: 0;
    left: 8px;
    height: 4px;
    cursor: ns-resize;
  }

  .resize-handle.east {
    top: 8px;
    right: 0;
    bottom: 8px;
    width: 4px;
    cursor: ew-resize;
  }

  .resize-handle.west {
    top: 8px;
    bottom: 8px;
    left: 0;
    width: 4px;
    cursor: ew-resize;
  }

  .resize-handle.northeast {
    top: 0;
    right: 0;
    width: 8px;
    height: 8px;
    cursor: nesw-resize;
  }

  .resize-handle.northwest {
    top: 0;
    left: 0;
    width: 8px;
    height: 8px;
    cursor: nwse-resize;
  }

  .resize-handle.southeast {
    right: 0;
    bottom: 0;
    width: 8px;
    height: 8px;
    cursor: nwse-resize;
  }

  .resize-handle.southwest {
    bottom: 0;
    left: 0;
    width: 8px;
    height: 8px;
    cursor: nesw-resize;
  }
</style>
