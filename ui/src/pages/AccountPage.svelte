<script lang="ts">
  import { onMount } from "svelte";
  import AuthForm from "../components/AuthForm.svelte";
  import Button from "../components/Button.svelte";
  import SkinViewer from "../components/SkinViewer.svelte";
  import { api, type SkinData } from "../lib/api";
  import { navigate } from "../lib/nav";
  import { snapshot, loadSnapshot } from "../lib/state";
  import { skin as skinStore, preloadSkin } from "../lib/skin";

  let changing = false;
  let busy = false;
  let error: string | null = null;

  let uploading = false;
  let actionError: string | null = null;
  let modelChoice: "default" | "slim" = "default";
  let confirmDelete = false;
  let fileInput: HTMLInputElement;
  let lastSkin: SkinData | null = null;
  let lastAccountUuid: string | null = null;

  $: account = $snapshot?.account ?? null;
  $: connected = !!$snapshot?.server;
  $: showForm = connected && (!account || changing);

  $: skin = $skinStore.data;
  $: skinLoading = $skinStore.loading;
  $: skinError = $skinStore.error ?? actionError;

  $: if (account?.uuid !== lastAccountUuid) {
    lastAccountUuid = account?.uuid ?? null;
    actionError = null;
    confirmDelete = false;
  }

  // Keep the selected model in sync with the preloaded skin without fighting
  // a manual toggle.
  $: if ($skinStore.data !== lastSkin) {
    lastSkin = $skinStore.data;
    confirmDelete = false;
    if ($skinStore.data) modelChoice = $skinStore.data.model;
  }

  onMount(() => {
    // The skin is normally preloaded already; only retry a failed fetch.
    if ($skinStore.error) void preloadSkin(true);
  });

  async function logout() {
    if (busy) return;
    busy = true;
    try {
      await api.logout();
      await loadSnapshot();
      changing = false;
    } catch (err) {
      error = String(err);
    } finally {
      busy = false;
    }
  }

  function pickSkin() {
    fileInput?.click();
  }

  function toBase64(buffer: ArrayBuffer): string {
    let binary = "";
    const bytes = new Uint8Array(buffer);
    const chunk = 0x8000;
    for (let i = 0; i < bytes.length; i += chunk) {
      binary += String.fromCharCode(...bytes.subarray(i, i + chunk));
    }
    return btoa(binary);
  }

  async function onSkinFile(event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    input.value = "";
    if (!file) return;
    if (file.type !== "image/png") {
      actionError = "请选择 PNG 皮肤文件";
      return;
    }
    if (file.size > 2 * 1024 * 1024) {
      actionError = "皮肤文件不能超过 2 MB";
      return;
    }

    uploading = true;
    actionError = null;
    confirmDelete = false;
    try {
      const data = toBase64(await file.arrayBuffer());
      await api.uploadSkin(data, null);
      await preloadSkin(true);
    } catch (err) {
      actionError = String(err);
    } finally {
      uploading = false;
    }
  }

  async function chooseModel(next: "default" | "slim") {
    if (modelChoice === next) return;
    const previous = modelChoice;
    modelChoice = next;
    if (!skin || uploading) return;

    uploading = true;
    actionError = null;
    try {
      const data = skin.dataUrl.slice(skin.dataUrl.indexOf(",") + 1);
      await api.uploadSkin(data, next);
      await preloadSkin(true);
    } catch (err) {
      modelChoice = previous;
      actionError = String(err);
    } finally {
      uploading = false;
    }
  }

  async function removeSkin() {
    if (!skin || uploading) return;
    if (!confirmDelete) {
      confirmDelete = true;
      return;
    }

    uploading = true;
    actionError = null;
    try {
      await api.deleteSkin();
      confirmDelete = false;
      await preloadSkin(true);
    } catch (err) {
      actionError = String(err);
    } finally {
      uploading = false;
    }
  }
</script>

<div class="page">
  {#if account}
    <div class="account-layout">
      <div class="account-main">
        <header class="account-head">
          <div class="profile">
            <div class="avatar" class:image={skin}>
              {#if skin}
                <img src={skin.avatarUrl} alt="" />
              {:else}
                {account.name.slice(0, 1).toUpperCase()}
              {/if}
            </div>
            <div>
              <h1 class="page-title">{account.name}</h1>
              <div class="profile-kind">
                {account.kind === "syncraft" ? "Syncraft 账号" : "离线账号"}
              </div>
            </div>
          </div>

          <div class="account-actions">
            {#if connected}
              <Button variant="signal" size="sm" onclick={() => (changing = !changing)}>
                更换账号
              </Button>
            {/if}
            <Button variant="signal" size="sm" onclick={logout} disabled={busy}>
              退出登录
            </Button>
          </div>
        </header>

        {#if error}
          <p class="message error" role="alert">{error}</p>
        {/if}

        <div class="details">
          {#if account.email}
            <div class="detail">
              <span class="detail-label">邮箱</span>
              <span class="detail-value mono">{account.email}</span>
            </div>
          {/if}
          <div class="detail">
            <span class="detail-label">UUID</span>
            <span class="detail-value mono">{account.uuid}</span>
          </div>
        </div>

        {#if showForm}
          <AuthForm attached onsuccess={() => (changing = false)} />
        {/if}
      </div>

      {#if connected && account.kind === "syncraft"}
        <aside class="skin-column">
          {#if skin}
            <SkinViewer skin={skin.dataUrl} model={modelChoice} />
          {:else}
            <div class="skin-placeholder">
              {#if skinLoading}
                <span class="loading" aria-label="正在载入皮肤"></span>
              {:else}
                <span aria-hidden="true">{account.name.slice(0, 1).toUpperCase()}</span>
              {/if}
            </div>
          {/if}

          <div class="skin-controls">
            <div class="models" role="group" aria-label="皮肤模型">
              <button
                type="button"
                class="model"
                class:active={modelChoice === "default"}
                onclick={() => chooseModel("default")}
              >
                经典
              </button>
              <button
                type="button"
                class="model"
                class:active={modelChoice === "slim"}
                onclick={() => chooseModel("slim")}
              >
                苗条
              </button>
            </div>

            <div class="skin-actions">
              <Button
                variant="outline"
                size="sm"
                disabled={uploading || skinLoading}
                onclick={pickSkin}
              >
                上传皮肤
              </Button>
              {#if skin}
                <Button
                  variant="signal"
                  size="sm"
                  disabled={uploading}
                  onclick={removeSkin}
                >
                  {confirmDelete ? "确认删除" : "删除皮肤"}
                </Button>
              {/if}
            </div>

            <input
              class="file"
              type="file"
              accept="image/png"
              bind:this={fileInput}
              onchange={onSkinFile}
            />

            {#if skinError}
              <p class="skin-error" role="alert">{skinError}</p>
            {/if}
          </div>
        </aside>
      {/if}
    </div>
  {:else}
    <h1 class="page-title">账号</h1>

    {#if error}
      <p class="message error" role="alert">{error}</p>
    {/if}

    {#if !connected}
      <div class="connect-prompt">
        <span>请先连接服务器</span>
        <Button variant="outline" size="sm" onclick={() => navigate("server")}>
          前往服务器
        </Button>
      </div>
    {/if}

    {#if showForm}
      <AuthForm onsuccess={() => (changing = false)} />
    {/if}
  {/if}
</div>

<style>
  .account-layout {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 320px;
    gap: var(--gutter);
    align-items: start;
  }

  .account-main {
    min-width: 0;
  }

  .account-head {
    display: flex;
    flex-wrap: wrap;
    align-items: flex-start;
    justify-content: space-between;
    gap: 24px;
  }

  .profile {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .avatar {
    display: flex;
    width: 64px;
    height: 64px;
    flex: none;
    align-items: center;
    justify-content: center;
    overflow: hidden;
    background: var(--ink);
    color: var(--paper);
    font-family: var(--font-display);
    font-size: 24px;
    font-weight: 400;
    line-height: 32px;
  }

  .avatar.image {
    background: transparent;
  }

  .avatar img {
    display: block;
    width: 100%;
    height: 100%;
    object-fit: contain;
  }

  .profile-kind {
    margin-top: 4px;
    color: var(--muted);
    font-size: 14px;
    line-height: 20px;
  }

  .account-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .message {
    margin: 24px 0 0;
    font-size: 14px;
    line-height: 22px;
    overflow-wrap: anywhere;
  }

  .message.error {
    color: var(--signal);
  }

  .connect-prompt {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 16px;
    margin-top: 32px;
    font-size: 14px;
    line-height: 22px;
  }

  .skin-column {
    display: flex;
    min-width: 0;
    flex-direction: column;
    gap: 12px;
  }

  .skin-placeholder {
    display: flex;
    width: 100%;
    aspect-ratio: 3 / 4;
    align-items: center;
    justify-content: center;
    border: 1px solid var(--rule);
    background: var(--paper);
    color: var(--muted);
    font-family: var(--font-display);
    font-size: 64px;
    line-height: 1;
  }

  .loading {
    display: block;
    width: 10px;
    height: 10px;
    background: var(--ink);
    animation: pulse 900ms ease-in-out infinite alternate;
  }

  @keyframes pulse {
    from {
      opacity: 0.2;
    }
    to {
      opacity: 1;
    }
  }

  .skin-controls {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .models {
    display: flex;
    gap: 4px;
  }

  .model {
    height: 32px;
    padding: 0 12px;
    border: 1px solid var(--rule);
    color: var(--muted);
    font-size: 14px;
    line-height: 20px;
    transition:
      background-color 160ms var(--ease),
      border-color 160ms var(--ease),
      color 160ms var(--ease);
  }

  .model:hover {
    border-color: var(--ink);
    color: var(--ink);
  }

  .model.active {
    border-color: var(--ink);
    background: var(--ink);
    color: var(--paper);
  }

  .skin-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .file {
    display: none;
  }

  .skin-error {
    margin: 0;
    color: var(--signal);
    font-size: 12px;
    line-height: 18px;
    overflow-wrap: anywhere;
  }

  @media (max-width: 820px) {
    .account-layout {
      grid-template-columns: minmax(0, 1fr);
      gap: 24px;
    }

    .profile {
      gap: 12px;
    }

    .skin-column {
      max-width: 320px;
    }
  }
</style>
