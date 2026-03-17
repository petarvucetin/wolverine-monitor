<script lang="ts">
  import type { SslMode } from "$lib/types";
  import { createConnection, testConnectionConfig } from "$lib/stores/connections";

  let name = $state("");
  let host = $state("localhost");
  let port = $state(5432);
  let database = $state("");
  let schema = $state("public");
  let username = $state("");
  let password = $state("");
  let sslMode = $state<SslMode>("Prefer");
  let testing = $state(false);
  let saving = $state(false);

  async function handleTest() {
    testing = true;
    await testConnectionConfig(host, port, database, username, password, sslMode);
    testing = false;
  }

  async function handleSave() {
    saving = true;
    try {
      await createConnection({ name, host, port, database, schema, username, password, ssl_mode: sslMode });
      // Reset form
      name = ""; database = ""; username = ""; password = "";
    } catch { /* toast already shown */ }
    saving = false;
  }
</script>

<form class="space-y-4 p-4 bg-[var(--color-surface-raised)] rounded-lg border border-[var(--color-border)]"
      onsubmit={(e) => { e.preventDefault(); handleSave(); }}>
  <h3 class="text-sm font-semibold">New Connection</h3>

  <div class="grid grid-cols-2 gap-3">
    <label class="block">
      <span class="text-xs text-[var(--color-text-secondary)]">Name</span>
      <input bind:value={name} required
        class="mt-1 w-full bg-[var(--color-surface)] border border-[var(--color-border)] rounded px-3 py-1.5 text-sm" />
    </label>
    <label class="block">
      <span class="text-xs text-[var(--color-text-secondary)]">Host</span>
      <input bind:value={host} required
        class="mt-1 w-full bg-[var(--color-surface)] border border-[var(--color-border)] rounded px-3 py-1.5 text-sm" />
    </label>
    <label class="block">
      <span class="text-xs text-[var(--color-text-secondary)]">Port</span>
      <input bind:value={port} type="number" required
        class="mt-1 w-full bg-[var(--color-surface)] border border-[var(--color-border)] rounded px-3 py-1.5 text-sm" />
    </label>
    <label class="block">
      <span class="text-xs text-[var(--color-text-secondary)]">Database</span>
      <input bind:value={database} required
        class="mt-1 w-full bg-[var(--color-surface)] border border-[var(--color-border)] rounded px-3 py-1.5 text-sm" />
    </label>
    <label class="block">
      <span class="text-xs text-[var(--color-text-secondary)]">Schema</span>
      <input bind:value={schema} required
        class="mt-1 w-full bg-[var(--color-surface)] border border-[var(--color-border)] rounded px-3 py-1.5 text-sm" />
    </label>
    <label class="block">
      <span class="text-xs text-[var(--color-text-secondary)]">Username</span>
      <input bind:value={username} required
        class="mt-1 w-full bg-[var(--color-surface)] border border-[var(--color-border)] rounded px-3 py-1.5 text-sm" />
    </label>
    <label class="block">
      <span class="text-xs text-[var(--color-text-secondary)]">Password</span>
      <input bind:value={password} type="password" required
        class="mt-1 w-full bg-[var(--color-surface)] border border-[var(--color-border)] rounded px-3 py-1.5 text-sm" />
    </label>
    <label class="block">
      <span class="text-xs text-[var(--color-text-secondary)]">SSL Mode</span>
      <select bind:value={sslMode}
        class="mt-1 w-full bg-[var(--color-surface)] border border-[var(--color-border)] rounded px-3 py-1.5 text-sm">
        <option value="Disable">Disable</option>
        <option value="Prefer">Prefer</option>
        <option value="Require">Require</option>
        <option value="VerifyCa">Verify CA</option>
      </select>
    </label>
  </div>

  <div class="flex gap-2">
    <button type="button" onclick={handleTest} disabled={testing}
      class="px-4 py-1.5 text-sm rounded border border-[var(--color-border)] hover:bg-[var(--color-surface-overlay)] disabled:opacity-50">
      {testing ? "Testing..." : "Test Connection"}
    </button>
    <button type="submit" disabled={saving}
      class="px-4 py-1.5 text-sm rounded bg-blue-600 hover:bg-blue-700 text-white disabled:opacity-50">
      {saving ? "Connecting..." : "Save & Connect"}
    </button>
  </div>
</form>
