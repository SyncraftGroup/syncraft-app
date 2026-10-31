<script lang="ts">
  import Button from "../components/Button.svelte";
  import TextField from "../components/TextField.svelte";
  import { api } from "../lib/api";
  import { loadSnapshot } from "../lib/state";

  type Mode = "login" | "register" | "offline";

  let {
    attached = false,
    onsuccess,
  }: {
    attached?: boolean;
    onsuccess?: () => void;
  } = $props();

  let mode = $state<Mode>("login");
  let busy = $state(false);
  let error = $state<string | null>(null);
  let notice = $state<string | null>(null);

  let email = $state("");
  let password = $state("");
  let username = $state("");
  let registerEmail = $state("");
  let registerPassword = $state("");

  function resetMessages() {
    error = null;
    notice = null;
  }

  function switchMode(next: Mode) {
    mode = next;
    resetMessages();
  }

  async function submitLogin(event: SubmitEvent) {
    event.preventDefault();
    if (busy) return;
    busy = true;
    resetMessages();
    try {
      await api.login(email, password);
      password = "";
      notice = null;
      await loadSnapshot();
      onsuccess?.();
    } catch (err) {
      error = String(err);
    } finally {
      busy = false;
    }
  }

  async function submitRegister(event: SubmitEvent) {
    event.preventDefault();
    if (busy) return;
    busy = true;
    resetMessages();
    try {
      await api.register(username, registerEmail, registerPassword);
      notice = "注册成功，请登录";
      email = registerEmail;
      password = "";
      mode = "login";
    } catch (err) {
      error = String(err);
    } finally {
      busy = false;
    }
  }

  async function submitOffline(event: SubmitEvent) {
    event.preventDefault();
    if (busy) return;
    busy = true;
    resetMessages();
    try {
      await api.setOfflineAccount(username);
      notice = null;
      await loadSnapshot();
      onsuccess?.();
    } catch (err) {
      error = String(err);
    } finally {
      busy = false;
    }
  }
</script>

<div class="auth" class:attached>
  {#if error}
    <p class="message error" role="alert">{error}</p>
  {:else if notice}
    <p class="message" role="status">{notice}</p>
  {/if}

  <div class="modes" role="tablist" aria-label="登录方式">
    <button
      type="button"
      class="mode"
      class:active={mode === "login"}
      onclick={() => switchMode("login")}
    >
      登录
    </button>
    <button
      type="button"
      class="mode"
      class:active={mode === "register"}
      onclick={() => switchMode("register")}
    >
      注册
    </button>
    <button
      type="button"
      class="mode"
      class:active={mode === "offline"}
      onclick={() => switchMode("offline")}
    >
      离线
    </button>
  </div>

  {#if mode === "login"}
    <form class="fields" onsubmit={submitLogin}>
      <TextField
        bind:value={email}
        type="email"
        ariaLabel="邮箱"
        placeholder="邮箱"
        autocomplete="username"
      />
      <TextField
        bind:value={password}
        type="password"
        ariaLabel="密码"
        placeholder="密码"
        autocomplete="current-password"
      />
      <div class="submit-row">
        <Button type="submit" variant="primary" size="sm" disabled={busy}>登录</Button>
      </div>
    </form>
  {:else if mode === "register"}
    <form class="fields" onsubmit={submitRegister}>
      <TextField
        bind:value={username}
        ariaLabel="角色名"
        placeholder="角色名"
        autocomplete="nickname"
      />
      <TextField
        bind:value={registerEmail}
        type="email"
        ariaLabel="邮箱"
        placeholder="邮箱"
        autocomplete="username"
      />
      <TextField
        bind:value={registerPassword}
        type="password"
        ariaLabel="密码"
        placeholder="密码（至少 8 位）"
        autocomplete="new-password"
      />
      <div class="submit-row">
        <Button type="submit" variant="primary" size="sm" disabled={busy}>注册</Button>
      </div>
    </form>
  {:else}
    <form class="fields" onsubmit={submitOffline}>
      <TextField
        bind:value={username}
        ariaLabel="角色名"
        placeholder="角色名"
        autocomplete="nickname"
      />
      <div class="submit-row">
        <Button type="submit" variant="primary" size="sm" disabled={busy}>
          使用离线账号
        </Button>
      </div>
    </form>
  {/if}
</div>

<style>
  .auth {
    margin-top: 32px;
  }

  .auth.attached {
    margin-top: 24px;
    padding-top: 24px;
    border-top: 1px solid var(--rule);
  }

  .message {
    margin: 0 0 20px;
    font-size: 14px;
    line-height: 22px;
    overflow-wrap: anywhere;
  }

  .message.error {
    color: var(--signal);
  }

  .modes {
    display: flex;
    gap: 4px;
  }

  .mode {
    height: 32px;
    padding: 0 12px;
    color: var(--muted);
    font-size: 14px;
    line-height: 20px;
    transition:
      background-color 160ms var(--ease),
      color 160ms var(--ease);
  }

  .mode:hover {
    color: var(--ink);
  }

  .mode.active {
    background: var(--ink);
    color: var(--paper);
  }

  .fields {
    display: grid;
    max-width: 360px;
    gap: 20px;
    margin-top: 24px;
  }

  .submit-row {
    display: flex;
    margin-top: 4px;
  }
</style>
