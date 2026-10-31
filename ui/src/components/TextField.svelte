<script lang="ts">
  let {
    value = $bindable(""),
    placeholder = "",
    ariaLabel,
    type = "text",
    autocomplete,
  }: {
    value?: string;
    placeholder?: string;
    ariaLabel?: string;
    type?: "text" | "url" | "email" | "password";
    autocomplete?: string;
  } = $props();
</script>

<label class="field">
  <input
    {type}
    bind:value
    {placeholder}
    aria-label={ariaLabel}
    {autocomplete}
  />
  <span class="line" aria-hidden="true"></span>
</label>

<style>
  .field {
    position: relative;
    display: block;
    width: 100%;
    padding-bottom: 7px;
    border-bottom: 1px solid var(--rule);
  }

  input {
    width: 100%;
    height: 30px;
    padding: 0;
    border: 0;
    outline: 0;
    background: transparent;
    color: var(--ink);
    font-size: 14px;
    line-height: 20px;
  }

  input::placeholder {
    color: var(--muted);
  }

  .line {
    position: absolute;
    right: 0;
    bottom: -1px;
    left: 0;
    height: 2px;
    background: var(--ink);
    transform: scaleX(0);
    transform-origin: left;
    transition: transform 220ms var(--ease);
  }

  .field:focus-within .line {
    transform: scaleX(1);
  }
</style>
