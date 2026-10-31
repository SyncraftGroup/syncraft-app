<script lang="ts">
  let {
    checked = $bindable(false),
    label,
    onchange,
  }: {
    checked?: boolean;
    label: string;
    onchange?: (value: boolean) => void;
  } = $props();

  function toggle() {
    checked = !checked;
    onchange?.(checked);
  }
</script>

<button
  type="button"
  role="switch"
  aria-checked={checked}
  aria-label={label}
  class="toggle"
  class:on={checked}
  onclick={toggle}
>
  <span class="knob"></span>
</button>

<style>
  .toggle {
    display: flex;
    width: 40px;
    height: 24px;
    flex: none;
    align-items: center;
    padding: 3px;
    border: 2px solid var(--ink);
    background: var(--paper);
    transition: background-color 200ms var(--ease);
  }

  .knob {
    display: block;
    width: 14px;
    height: 14px;
    background: var(--ink);
    transition:
      transform 220ms var(--ease),
      background-color 200ms var(--ease);
  }

  .toggle.on {
    background: var(--ink);
  }

  .toggle.on .knob {
    background: var(--paper);
    transform: translateX(16px);
  }
</style>
