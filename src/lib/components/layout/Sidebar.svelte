<script lang="ts">
  import { currentRoute, navigate } from "$lib/stores/router";
  import { connections, activeConnectionId } from "$lib/stores/connections";
  import type { Route } from "$lib/types";

  const navItems: { route: Route; label: string; icon: string }[] = [
    { route: "dashboard", label: "Dashboard", icon: "\u25C9" },
    { route: "explorer", label: "Explorer", icon: "\u229E" },
    { route: "deadletters", label: "Dead Letters", icon: "\u26A0" },
    { route: "nodes", label: "Nodes", icon: "\u2B21" },
    { route: "connections", label: "Connections", icon: "\u26C1" },
  ];

  function handleConnectionChange(e: Event) {
    const target = e.target as HTMLSelectElement;
    activeConnectionId.set(target.value || null);
  }
</script>

<aside class="w-56 h-screen flex flex-col bg-[var(--color-surface-raised)] border-r border-[var(--color-border)]">
  <div class="px-4 py-5 text-lg font-bold tracking-tight">
    Wolverine Monitor
  </div>

  {#if $connections.length > 0}
    <div class="px-3 pb-3">
      <select
        class="w-full px-2 py-1.5 text-sm rounded-md bg-[var(--color-surface)] border border-[var(--color-border)] text-[var(--color-text-primary)] focus:outline-none focus:border-blue-500"
        value={$activeConnectionId ?? ""}
        onchange={handleConnectionChange}
      >
        <option value="">Select connection...</option>
        {#each $connections as conn}
          <option value={conn.config.id}>{conn.config.name}</option>
        {/each}
      </select>
    </div>
  {/if}

  <nav class="flex-1 px-2 space-y-1">
    {#each navItems as item}
      <button
        class="w-full flex items-center gap-3 px-3 py-2 rounded-md text-sm transition-colors
          {$currentRoute === item.route
            ? 'bg-[var(--color-surface-overlay)] text-white'
            : 'text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-overlay)] hover:text-white'}"
        onclick={() => navigate(item.route)}
      >
        <span class="text-base">{item.icon}</span>
        {item.label}
      </button>
    {/each}
  </nav>

  <div class="px-4 py-3 text-xs text-[var(--color-text-secondary)] border-t border-[var(--color-border)]">
    v0.1.0
  </div>
</aside>
