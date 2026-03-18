<script lang="ts">
  import type { Route, SslMode } from "$lib/types";
  import { createConnection, testConnectionConfig } from "$lib/stores/connections";

  let name = $state("");
  let host = $state("localhost");
  let port = $state(5432);
  let database = $state("");
  let schema = $state("public");
  let tablePrefix = $state("wolverine_");
  let username = $state("");
  let password = $state("");
  let sslMode = $state<SslMode>("Prefer");
  let testing = $state(false);
  let saving = $state(false);

  let selectedRoutes = $state<Record<string, boolean>>({
    dashboard: false,
    explorer: false,
    deadletters: false,
    nodes: false,
    queues: false,
  });

  const routeLabels: { route: Route; label: string }[] = [
    { route: "dashboard", label: "Dashboard" },
    { route: "explorer", label: "Explorer" },
    { route: "deadletters", label: "Dead Letters" },
    { route: "nodes", label: "Nodes" },
    { route: "queues", label: "Queues" },
  ];

  function getRoutes(): Route[] {
    return Object.entries(selectedRoutes)
      .filter(([, v]) => v)
      .map(([k]) => k as Route);
  }

  async function handleTest() {
    testing = true;
    await testConnectionConfig(host, port, database, username, password, sslMode);
    testing = false;
  }

  async function handleSave() {
    saving = true;
    try {
      await createConnection({ name, routes: getRoutes(), host, port, database, schema, table_prefix: tablePrefix, username, password, ssl_mode: sslMode });
      name = ""; database = ""; username = ""; password = "";
      selectedRoutes = { dashboard: false, explorer: false, deadletters: false, nodes: false };
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
      <span class="text-xs text-[var(--color-text-secondary)]">Table Prefix</span>
      <input bind:value={tablePrefix} placeholder="wolverine_"
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

  <div>
    <span class="text-xs text-[var(--color-text-secondary)]">Use for pages</span>
    <div class="flex gap-4 mt-1.5">
      {#each routeLabels as { route, label }}
        <label class="flex items-center gap-1.5 text-sm cursor-pointer">
          <input type="checkbox" bind:checked={selectedRoutes[route]}
            class="rounded border-[var(--color-border)]" />
          {label}
        </label>
      {/each}
    </div>
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
