<script lang="ts">
  import Button from "../components/Button.svelte";
  import TextField from "../components/TextField.svelte";
  import Toggle from "../components/Toggle.svelte";
  import { api } from "../lib/api";
  import { snapshot, loadSnapshot, patchSnapshot } from "../lib/state";

  let editingDir = false;
  let editingJava = false;
  let editingSource = false;
  let editingConcurrency = false;
  let dirDraft = "";
  let javaDraft = "";
  let busy = false;
  let error: string | null = null;

  let minimize = false;
  let autoLaunch = false;

  $: state = $snapshot;
  $: minimize = state?.minimizeOnLaunch ?? false;
  $: autoLaunch = state?.autoLaunch ?? false;
  $: javaLabel = state?.java
    ? state.java.version > 0
      ? `Java ${state.java.version}`
      : state.java.path.split(/[\\/]/).pop() || "自定义"
    : "自动";
  $: sourceLabel = state?.source === "bmclapi" ? "BMCLAPI" : "官方";
  $: concurrencyLabel = state?.concurrency ? `${state.concurrency}` : "自动";

  function beginDir() {
    dirDraft = state?.gameDir ?? "";
    editingDir = true;
    error = null;
  }

  async function saveDir(event: SubmitEvent) {
    event.preventDefault();
    if (busy) return;
    busy = true;
    error = null;
    try {
      const dir = await api.setGameDir(dirDraft);
      patchSnapshot({ gameDir: dir });
      editingDir = false;
    } catch (err) {
      error = String(err);
    } finally {
      busy = false;
    }
  }

  function beginJava() {
    javaDraft = state?.java?.path ?? "";
    editingJava = !editingJava;
    error = null;
  }

  async function chooseJava(path: string | null) {
    if (busy) return;
    busy = true;
    error = null;
    try {
      await api.setJava(path);
      await loadSnapshot();
      editingJava = false;
    } catch (err) {
      error = String(err);
    } finally {
      busy = false;
    }
  }

  async function chooseSource(source: "default" | "bmclapi") {
    if (busy) return;
    busy = true;
    error = null;
    try {
      await api.setSource(source);
      patchSnapshot({ source });
      editingSource = false;
    } catch (err) {
      error = String(err);
    } finally {
      busy = false;
    }
  }

  async function chooseConcurrency(value: number | null) {
    if (busy) return;
    busy = true;
    error = null;
    try {
      await api.setConcurrency(value);
      patchSnapshot({ concurrency: value });
      editingConcurrency = false;
    } catch (err) {
      error = String(err);
    } finally {
      busy = false;
    }
  }

  async function toggleMinimize(value: boolean) {
    patchSnapshot({ minimizeOnLaunch: value });
    try {
      await api.setToggle("minimizeOnLaunch", value);
    } catch (err) {
      error = String(err);
    }
  }

  async function toggleAutoLaunch(value: boolean) {
    patchSnapshot({ autoLaunch: value });
    try {
      await api.setToggle("autoLaunch", value);
    } catch (err) {
      error = String(err);
    }
  }
</script>

<div class="page">
  <h1 class="page-title">设置</h1>

  {#if error}
    <p class="message error" role="alert">{error}</p>
  {/if}

  <div class="settings-grid">
    <section class="section">
      <h2 class="section-title">游戏</h2>
      <div class="settings">
        <button class="setting-row" type="button" onclick={beginDir}>
          <span class="setting-label">游戏目录</span>
          <span class="setting-value mono">{state?.gameDir ?? "—"}</span>
          <span class="chevron" aria-hidden="true"></span>
        </button>
        {#if editingDir}
          <form class="inline-form" onsubmit={saveDir}>
            <div class="field-wrap">
              <TextField
                bind:value={dirDraft}
                ariaLabel="游戏目录"
                placeholder="/path/to/.minecraft"
              />
            </div>
            <Button type="submit" variant="primary" size="sm" disabled={busy}>保存</Button>
            <Button variant="ghost" size="sm" onclick={() => (editingDir = false)}>取消</Button>
          </form>
        {/if}

        <button class="setting-row" type="button" onclick={beginJava}>
          <span class="setting-label">运行时</span>
          <span class="setting-value">{javaLabel}</span>
          <span class="chevron" aria-hidden="true"></span>
        </button>
        {#if editingJava}
          <div class="options">
            <button
              type="button"
              class="option"
              class:selected={!state?.java}
              onclick={() => chooseJava(null)}
            >
              自动
            </button>
            {#each state?.detectedJava ?? [] as java}
              <button
                type="button"
                class="option"
                class:selected={java.selected}
                onclick={() => chooseJava(java.path)}
              >
                <span>Java {java.version}</span>
                <span class="option-path mono">{java.path}</span>
              </button>
            {/each}
            <div class="manual">
              <div class="field-wrap">
                <TextField
                  bind:value={javaDraft}
                  ariaLabel="Java 路径"
                  placeholder="Java 可执行文件路径"
                />
              </div>
              <Button
                variant="outline"
                size="sm"
                disabled={busy || !javaDraft.trim()}
                onclick={() => chooseJava(javaDraft.trim())}
              >
                使用
              </Button>
            </div>
          </div>
        {/if}

        <button class="setting-row" type="button" onclick={() => (editingSource = !editingSource)}>
          <span class="setting-label">下载源</span>
          <span class="setting-value">{sourceLabel}</span>
          <span class="chevron" aria-hidden="true"></span>
        </button>
        {#if editingSource}
          <div class="options">
            <button
              type="button"
              class="option"
              class:selected={state?.source !== "bmclapi"}
              onclick={() => chooseSource("default")}
            >
              官方
            </button>
            <button
              type="button"
              class="option"
              class:selected={state?.source === "bmclapi"}
              onclick={() => chooseSource("bmclapi")}
            >
              BMCLAPI
            </button>
          </div>
        {/if}

        <button
          class="setting-row"
          type="button"
          onclick={() => (editingConcurrency = !editingConcurrency)}
        >
          <span class="setting-label">并发下载</span>
          <span class="setting-value mono">{concurrencyLabel}</span>
          <span class="chevron" aria-hidden="true"></span>
        </button>
        {#if editingConcurrency}
          <div class="options">
            {#each [null, 4, 8, 12, 16] as value}
              <button
                type="button"
                class="option"
                class:selected={state?.concurrency === value}
                onclick={() => chooseConcurrency(value)}
              >
                {value ?? "自动"}
              </button>
            {/each}
          </div>
        {/if}
      </div>
    </section>

    <div class="column">
      <section class="section">
        <h2 class="section-title">启动</h2>
        <div class="settings">
          <div class="setting-row static">
            <span class="setting-label">启动后最小化</span>
            <div class="setting-control">
              <Toggle
                bind:checked={minimize}
                label="启动后最小化"
                onchange={toggleMinimize}
              />
            </div>
          </div>
          <div class="setting-row static">
            <span class="setting-label">自动开始游戏</span>
            <div class="setting-control">
              <Toggle
                bind:checked={autoLaunch}
                label="自动开始游戏"
                onchange={toggleAutoLaunch}
              />
            </div>
          </div>
        </div>
      </section>

      <section class="section">
        <h2 class="section-title">关于</h2>
        <div class="settings">
          <div class="setting-row static">
            <span class="setting-label">版本</span>
            <span class="setting-value mono">{state?.version ?? "0.1.0"}</span>
          </div>
        </div>
      </section>
    </div>
  </div>
</div>

<style>
  .settings-grid {
    display: grid;
    grid-template-columns: minmax(0, 1.08fr) minmax(0, 0.92fr);
    gap: var(--gutter);
    margin-top: var(--section-gap);
  }

  .settings-grid .section {
    margin-top: 0;
  }

  .column .section + .section {
    margin-top: var(--section-gap);
  }

  .settings {
    margin-top: 2px;
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

  .inline-form {
    display: flex;
    flex-wrap: wrap;
    align-items: flex-end;
    gap: 8px;
    padding: 12px 0;
    border-bottom: 1px solid var(--rule);
  }

  .field-wrap {
    min-width: 200px;
    flex: 1 1 220px;
  }

  .options {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    padding: 10px 0;
    border-bottom: 1px solid var(--rule);
  }

  .option {
    display: inline-flex;
    max-width: 100%;
    height: 32px;
    align-items: center;
    gap: 8px;
    padding: 0 10px;
    border: 1px solid var(--rule);
    color: var(--ink);
    font-size: 14px;
    line-height: 20px;
    transition:
      background-color 160ms var(--ease),
      border-color 160ms var(--ease),
      color 160ms var(--ease);
  }

  .option:hover {
    border-color: var(--ink);
  }

  .option.selected {
    border-color: var(--ink);
    background: var(--ink);
    color: var(--paper);
  }

  .option-path {
    overflow: hidden;
    max-width: 220px;
    text-overflow: ellipsis;
    white-space: nowrap;
    opacity: 0.7;
  }

  .manual {
    display: flex;
    width: 100%;
    flex-wrap: wrap;
    align-items: flex-end;
    gap: 8px;
    margin-top: 6px;
  }

  @media (max-width: 820px) {
    .settings-grid {
      grid-template-columns: minmax(0, 1fr);
      margin-top: 24px;
    }

    .column .section + .section {
      margin-top: var(--section-gap);
    }

    .option-path {
      max-width: 140px;
    }
  }
</style>
