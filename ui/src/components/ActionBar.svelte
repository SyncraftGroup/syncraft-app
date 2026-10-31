<script lang="ts">
  import { page, navigate } from "../lib/nav";
  import { launch, startLaunch } from "../lib/launch";
  import { snapshot } from "../lib/state";
  import { formatBytes, formatSpeed } from "../lib/format";

  $: account = $snapshot?.account ?? null;
  $: server = $snapshot?.server ?? null;
  $: busy = ["checking", "installing", "syncing", "launching"].includes($launch.phase);
  $: running = $launch.phase === "running";
  $: failed = $launch.phase === "error";
  $: visible = $page === "home" || busy || running || failed;
  $: label = running
    ? "游戏运行中"
    : !server
      ? "连接服务器"
      : !account
        ? "添加账号"
        : failed
          ? "重试"
          : "开始游戏";
  $: stats =
    $launch.filesTotal > 0
      ? `${$launch.filesDone} / ${$launch.filesTotal} · ${formatBytes(
          $launch.bytesDone,
        )} / ${formatBytes($launch.bytesTotal)} · ${formatSpeed($launch.speed)}`
      : "";

  function handleClick() {
    if (!server) {
      navigate("server");
      return;
    }
    if (!account) {
      navigate("account");
      return;
    }
    void startLaunch();
  }
</script>

{#if visible}
  <div class="actionbar" class:live={busy || failed || running}>
    <div class="status" class:error={failed}>
      <span class="stage" aria-live="polite">{$launch.error ?? $launch.stage}</span>
      <span class="stats mono">{busy ? stats : ""}</span>
    </div>

    <button
      type="button"
      class="cta"
      class:running={busy}
      class:done={running}
      style={`--progress:${$launch.progress}%`}
      disabled={busy || running}
      aria-label={busy ? $launch.stage : label}
      onclick={handleClick}
    >
      <span class="fill"></span>
      <span class="label">{label}</span>
      <span class="arrow" aria-hidden="true"></span>
    </button>

    {#if busy}
      <span
        class="sr-only"
        role="progressbar"
        aria-valuemin="0"
        aria-valuemax="100"
        aria-valuenow={Math.round($launch.progress)}
        aria-label={$launch.stage}
      ></span>
    {/if}
  </div>
{/if}

<style>
  .actionbar {
    display: flex;
    height: var(--action-h);
    flex: 0 0 var(--action-h);
    flex-direction: column;
    justify-content: flex-end;
    gap: 8px;
    padding: 0 var(--gutter) var(--action-pad-bottom);
    border-top: 1px solid var(--rule);
    background: var(--paper);
  }

  .status {
    display: flex;
    height: 20px;
    align-items: center;
    justify-content: space-between;
    gap: 24px;
    opacity: 0;
    transition: opacity 160ms var(--ease);
  }

  .actionbar.live .status {
    opacity: 1;
  }

  .stage {
    overflow: hidden;
    font-size: 14px;
    letter-spacing: 0;
    line-height: 20px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .stats {
    flex: none;
    color: var(--muted);
    font-size: 12px;
    letter-spacing: 0;
    line-height: 20px;
    white-space: nowrap;
  }

  .status.error .stage {
    color: var(--signal);
  }

  .cta {
    position: relative;
    display: flex;
    width: 100%;
    height: 64px;
    flex: none;
    align-items: center;
    justify-content: space-between;
    padding: 0 24px;
    overflow: hidden;
    background: var(--ink);
    color: var(--paper);
    transition:
      height 260ms var(--ease),
      padding 260ms var(--ease),
      background-color 200ms var(--ease),
      opacity 200ms var(--ease);
  }

  .cta:hover:not(:disabled) {
    background: #000;
  }

  .cta:active:not(:disabled) {
    transform: translateY(1px);
  }

  .cta:focus-visible {
    outline-offset: 4px;
  }

  .label {
    position: relative;
    z-index: 2;
    font-family: var(--font-display);
    font-size: 22px;
    font-weight: 400;
    letter-spacing: 0;
    line-height: 30px;
    transition: opacity 120ms var(--ease);
  }

  .arrow {
    position: relative;
    z-index: 2;
    width: 24px;
    height: 16px;
    background: currentColor;
    clip-path: polygon(
      0 6px,
      14px 6px,
      14px 0,
      24px 8px,
      14px 16px,
      14px 10px,
      0 10px
    );
    opacity: 0.7;
    transition:
      opacity 200ms var(--ease),
      transform 200ms var(--ease);
  }

  .cta:hover:not(:disabled) .arrow {
    opacity: 1;
    transform: translateX(4px);
  }

  .fill {
    position: absolute;
    inset: 0 auto 0 0;
    width: var(--progress, 0%);
    background: var(--ink);
    opacity: 0;
    transition: opacity 120ms var(--ease);
  }

  .cta.running {
    height: 8px;
    padding: 0;
    background: var(--track);
    cursor: default;
  }

  .cta.running .fill {
    opacity: 1;
  }

  .cta.running .label,
  .cta.running .arrow {
    opacity: 0;
  }

  .cta.done {
    height: 0;
    padding: 0;
    opacity: 0;
    pointer-events: none;
  }

  @media (max-width: 820px) {
    .stats {
      font-size: 12px;
    }
  }
</style>
