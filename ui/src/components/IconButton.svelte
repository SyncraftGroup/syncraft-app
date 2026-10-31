<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    label,
    type = "button",
    disabled = false,
    onclick,
    children,
  }: {
    label: string;
    type?: "button" | "submit" | "reset";
    disabled?: boolean;
    onclick?: (event: MouseEvent) => void;
    children: Snippet;
  } = $props();
</script>

<button {type} {disabled} class="icon-btn" aria-label={label} title={label} {onclick}>
  {@render children()}
</button>

<style>
  .icon-btn {
    display: inline-flex;
    width: 32px;
    height: 32px;
    flex: none;
    align-items: center;
    justify-content: center;
    border: 2px solid var(--ink);
    background: var(--paper);
    color: var(--ink);
    transition:
      background-color 160ms var(--ease),
      color 160ms var(--ease),
      transform 100ms var(--ease);
  }

  .icon-btn:hover {
    background: var(--ink);
    color: var(--paper);
  }

  .icon-btn:active {
    transform: translateY(1px);
  }

  .icon-btn:disabled {
    cursor: default;
    opacity: 0.35;
  }

  .icon-btn:disabled:hover {
    background: var(--paper);
    color: var(--ink);
  }

  .icon-btn :global(svg) {
    width: 16px;
    height: 16px;
    fill: none;
    stroke: currentColor;
    stroke-linecap: square;
    stroke-linejoin: miter;
    stroke-width: 2;
    shape-rendering: crispEdges;
  }
</style>
