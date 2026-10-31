<script lang="ts">
  import { onMount, tick } from "svelte";
  import { isTauri } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import Logo from "./Logo.svelte";
  import { navigate, page, type Page } from "../lib/nav";
  import { maximized, refreshWindowState } from "../lib/window";

  const items: { id: Page; label: string }[] = [
    { id: "home", label: "主页" },
    { id: "server", label: "服务器" },
    { id: "account", label: "账号" },
    { id: "settings", label: "设置" },
  ];

  const appWindow = isTauri() ? getCurrentWindow() : null;

  let navEl: HTMLElement;
  let slots: { left: number; width: number }[] = [];
  let ready = false;

  async function measure() {
    await tick();
    if (!navEl) return;

    // Cache every item's geometry once. Page switches then only read this
    // array, avoiding a synchronous layout on the navigation critical path.
    const buttons = Array.from(navEl.querySelectorAll<HTMLElement>(".nav-item"));
    slots = buttons.map((button) => ({
      left: button.offsetLeft,
      width: button.offsetWidth,
    }));
    ready = true;
  }

  async function minimizeWindow() {
    await appWindow?.minimize();
  }

  async function toggleMaximizeWindow() {
    if (!appWindow) return;
    await appWindow.toggleMaximize();
    await refreshWindowState();
  }

  async function closeWindow() {
    await appWindow?.close();
  }

  $: capsule = slots[items.findIndex((item) => item.id === $page)] ?? {
    left: 0,
    width: 0,
  };

  onMount(() => {
    void measure();

    const observer = new ResizeObserver(() => void measure());
    if (navEl) observer.observe(navEl);
    const onResize = () => void measure();
    window.addEventListener("resize", onResize);

    if (document.fonts) {
      document.fonts.ready.then(() => void measure());
    }

    return () => {
      observer.disconnect();
      window.removeEventListener("resize", onResize);
    };
  });
</script>

<header class="topbar">
  <div class="drag-region" data-tauri-drag-region></div>

  <nav class="nav" bind:this={navEl} aria-label="主导航">
    {#each items as item}
      <button
        type="button"
        class="nav-item"
        aria-current={$page === item.id ? "page" : undefined}
        onclick={() => navigate(item.id)}
      >
        {#if item.id === "home"}
          <span class="nav-logo"><Logo /></span>
        {/if}
        <span class="nav-label">{item.label}</span>
      </button>
    {/each}

    <span
      class="indicator"
      class:ready
      style:transform={`translateX(${capsule.left}px)`}
      style:width={`${capsule.width}px`}
      aria-hidden="true"
    ></span>
  </nav>

  <div class="window-controls" role="group" aria-label="窗口控制">
    <button
      type="button"
      class="win-btn"
      aria-label="最小化"
      title="最小化"
      onclick={minimizeWindow}
    >
      <svg viewBox="0 0 16 16" aria-hidden="true">
        <path d="M3 7h10v2H3z" />
      </svg>
    </button>

    <button
      type="button"
      class="win-btn"
      aria-label={$maximized ? "还原" : "最大化"}
      title={$maximized ? "还原" : "最大化"}
      onclick={toggleMaximizeWindow}
    >
      <svg viewBox="0 0 16 16" aria-hidden="true">
        <path fill-rule="evenodd" d="M3 3h10v10H3V3Zm2 2v6h6V5H5Z" />
      </svg>
    </button>

    <button
      type="button"
      class="win-btn close"
      aria-label="关闭"
      title="关闭"
      onclick={closeWindow}
    >
      <svg viewBox="0 0 16 16" aria-hidden="true">
        <path
          d="M3 3h2v2H3zM11 3h2v2h-2zM5 5h2v2H5zM9 5h2v2H9zM7 7h2v2H7zM5 9h2v2H5zM9 9h2v2H9zM3 11h2v2H3zM11 11h2v2h-2z"
        />
      </svg>
    </button>
  </div>
</header>

<style>
  .topbar {
    position: relative;
    display: flex;
    /* The shell's 5px top border already occupies the top rule, so the header
       is 5px shorter; centering the 32px nav leaves equal 5px gaps above
       (against the rule) and below (against the header hairline). */
    height: calc(var(--topbar-h) - var(--window-edge));
    flex: 0 0 calc(var(--topbar-h) - var(--window-edge));
    align-items: center;
    padding: 0 0 0 var(--titlebar-pad);
    border-bottom: 1px solid var(--rule);
    background: var(--paper);
  }

  .drag-region {
    position: absolute;
    inset: 0;
    z-index: 0;
  }

  .nav {
    position: relative;
    z-index: 1;
    display: flex;
    height: 32px;
    align-items: center;
    gap: 4px;
    background: var(--paper);
    isolation: isolate;
  }

  .nav-item {
    display: flex;
    height: 32px;
    align-items: center;
    gap: 8px;
    padding: 0 12px;
    font-size: 14px;
    line-height: 20px;
    white-space: nowrap;
  }

  .nav-logo {
    display: flex;
    align-items: center;
    transition: opacity 160ms var(--ease);
  }

  .nav-label {
    transition: opacity 160ms var(--ease);
  }

  .nav-item:hover .nav-label,
  .nav-item:hover .nav-logo {
    opacity: 0.55;
  }

  .nav-item[aria-current="page"]:hover .nav-label,
  .nav-item[aria-current="page"]:hover .nav-logo {
    opacity: 1;
  }

  .indicator {
    position: absolute;
    top: 0;
    left: 0;
    z-index: 1;
    height: 32px;
    background: #fff;
    mix-blend-mode: difference;
    opacity: 0;
    pointer-events: none;
    transition:
      transform 320ms var(--ease),
      width 320ms var(--ease),
      opacity 180ms var(--ease);
  }

  .indicator.ready {
    opacity: 1;
  }

  .window-controls {
    position: relative;
    z-index: 1;
    display: flex;
    height: 100%;
    align-items: stretch;
    margin-left: auto;
    padding: 0 var(--controls-pad);
  }

  .win-btn {
    display: flex;
    width: 40px;
    height: 100%;
    align-items: center;
    justify-content: center;
    color: var(--ink);
    transition:
      background-color 160ms var(--ease),
      color 160ms var(--ease);
  }

  .win-btn:hover {
    background: var(--ink);
    color: var(--paper);
  }

  .win-btn.close:hover {
    background: var(--signal);
    color: #fff;
  }

  .win-btn svg {
    width: 16px;
    height: 16px;
    fill: currentColor;
    shape-rendering: crispEdges;
  }

  @media (max-width: 820px) {
    .nav {
      gap: 2px;
    }

    .nav-item {
      padding: 0 10px;
    }

    .win-btn {
      width: 34px;
    }
  }
</style>
