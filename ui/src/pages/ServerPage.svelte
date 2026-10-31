<script lang="ts">
  import { onMount } from "svelte";
  import { cubicOut } from "svelte/easing";
  import { slide } from "svelte/transition";
  import Button from "../components/Button.svelte";
  import IconButton from "../components/IconButton.svelte";
  import TextField from "../components/TextField.svelte";
  import { snapshot, serverError, refreshServer, connectServer } from "../lib/state";

  let editing = false;
  let link = "";
  let busy = false;
  let localError: string | null = null;

  $: server = $snapshot?.server ?? null;
  $: error = localError ?? $serverError;
  $: showForm = !server || editing;

  onMount(() => {
    link = $snapshot?.link ?? "";
    if ($snapshot?.link) void refreshServer();
  });

  function beginEdit() {
    link = $snapshot?.link ?? server?.link ?? "";
    editing = true;
    localError = null;
  }

  async function save(event: SubmitEvent) {
    event.preventDefault();
    busy = true;
    localError = null;
    const ok = await connectServer(link);
    busy = false;
    if (ok) editing = false;
  }

  async function refresh() {
    if (busy) return;
    busy = true;
    localError = null;
    await refreshServer();
    busy = false;
  }
</script>

<div class="page">
  <header class="server-head">
    <h1 class="page-title">{server?.name ?? "未连接服务器"}</h1>
    {#if server}
      <div class="head-actions">
        <IconButton label="更换服务器链接" onclick={beginEdit} disabled={busy}>
          <svg viewBox="0 0 20 20" aria-hidden="true">
            <path d="M4.5 15.5h3L15.5 7.5l-3-3L4.5 12.5v3Z" />
            <path d="m10.5 6.5 3 3" />
          </svg>
        </IconButton>
        <IconButton label="检查更新" onclick={refresh} disabled={busy}>
          <svg viewBox="0 0 20 20" aria-hidden="true">
            <path d="M16 10a6 6 0 1 1-1.8-4.3" />
            <path d="M16 4v4h-4" />
          </svg>
        </IconButton>
      </div>
    {/if}
  </header>

  {#if server}
    <div class="specs">
      <span class="spec-version mono">{server.version}</span>
      <span>
        {server.loader}
        {#if server.loaderVersion}<span class="mono">{server.loaderVersion}</span>{/if}
      </span>
      <span class="mono">{server.fileCount} 个文件</span>
    </div>
  {/if}

  {#if error}
    <p class="error" role="alert">{error}</p>
  {/if}

  {#if showForm}
    <form
      class="link-editor"
      transition:slide={{ duration: 220, easing: cubicOut }}
      onsubmit={save}
    >
      <div class="field-wrap">
        <TextField
          bind:value={link}
          ariaLabel="服务器链接"
          placeholder="服务器地址"
          autocomplete="url"
        />
      </div>
      <Button type="submit" variant="primary" size="sm" disabled={busy}>
        {server ? "保存" : "连接"}
      </Button>
      {#if server}
        <Button variant="ghost" size="sm" onclick={() => (editing = false)}>取消</Button>
      {/if}
    </form>
  {/if}

  {#if server && server.updates.length > 0}
    <section class="section">
      <div class="section-head">
        <h2 class="section-title">更新</h2>
      </div>

      <div class="updates">
        {#each server.updates as update}
          <p class="update-item">{update}</p>
        {/each}
      </div>
    </section>
  {/if}
</div>

<style>
  .server-head {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 24px;
  }

  .head-actions {
    display: flex;
    flex: none;
    align-items: center;
    gap: 8px;
  }

  .specs {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: 8px 24px;
    margin-top: 16px;
    color: var(--muted);
    font-size: 14px;
    line-height: 20px;
  }

  .spec-version {
    color: var(--ink);
    font-family: var(--font-data);
    font-size: 24px;
    font-weight: 400;
    line-height: 28px;
  }

  .error {
    margin: 20px 0 0;
    color: var(--signal);
    font-size: 14px;
    line-height: 22px;
    overflow-wrap: anywhere;
  }

  .link-editor {
    display: flex;
    flex-wrap: wrap;
    align-items: flex-end;
    gap: 8px;
    margin-top: 24px;
  }

  .field-wrap {
    min-width: 220px;
    flex: 1 1 260px;
  }

  .updates {
    margin-top: 4px;
  }

  .update-item {
    margin: 0;
    padding: 12px 0;
    border-bottom: 1px solid var(--rule);
    font-size: 14px;
    line-height: 22px;
  }

  .update-item:last-child {
    border-bottom: 0;
  }

  @media (max-width: 820px) {
    .server-head {
      gap: 16px;
    }

    .specs {
      gap: 8px 16px;
    }
  }
</style>
