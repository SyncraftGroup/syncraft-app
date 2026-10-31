<script lang="ts">
  import type { Snippet } from "svelte";

  type Variant = "primary" | "outline" | "signal" | "ghost";
  type Size = "sm" | "md";

  let {
    variant = "primary",
    size = "md",
    type = "button",
    disabled = false,
    ariaLabel,
    onclick,
    children,
  }: {
    variant?: Variant;
    size?: Size;
    type?: "button" | "submit" | "reset";
    disabled?: boolean;
    ariaLabel?: string;
    onclick?: (event: MouseEvent) => void;
    children: Snippet;
  } = $props();
</script>

<button
  {type}
  {disabled}
  class="btn {variant} {size}"
  aria-label={ariaLabel}
  {onclick}
>
  {@render children()}
</button>

<style>
  .btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border: 2px solid transparent;
    font-size: 14px;
    font-weight: 400;
    line-height: 20px;
    white-space: nowrap;
    transition:
      background-color 160ms var(--ease),
      border-color 160ms var(--ease),
      color 160ms var(--ease),
      transform 100ms var(--ease);
  }

  .btn:active:not(:disabled) {
    transform: translateY(1px);
  }

  .btn:disabled {
    cursor: default;
    opacity: 0.35;
  }

  .sm {
    height: 32px;
    padding: 0 12px;
  }

  .md {
    height: 36px;
    padding: 0 16px;
  }

  .primary {
    border-color: var(--ink);
    background: var(--ink);
    color: var(--paper);
  }

  .primary:hover:not(:disabled) {
    border-color: #000;
    background: #000;
  }

  .outline {
    border-color: var(--ink);
    background: transparent;
    color: var(--ink);
  }

  .outline:hover:not(:disabled) {
    background: var(--ink);
    color: var(--paper);
  }

  .signal {
    border-color: var(--signal);
    background: transparent;
    color: var(--signal);
  }

  .signal:hover:not(:disabled) {
    background: var(--signal);
    color: #fff;
  }

  .ghost {
    border-color: transparent;
    background: transparent;
    color: var(--muted);
  }

  .ghost:hover:not(:disabled) {
    color: var(--ink);
  }
</style>
