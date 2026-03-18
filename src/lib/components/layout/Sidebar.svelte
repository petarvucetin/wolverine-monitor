<script lang="ts">
  import { currentRoute, navigate } from "$lib/stores/router";
  import { connections } from "$lib/stores/connections";
  import { getVersion } from "@tauri-apps/api/app";
  import type { Route } from "$lib/types";

  let version = $state("...");
  getVersion().then((v) => (version = v));

  const navItems: { route: Route; label: string; icon: string }[] = [
    { route: "dashboard", label: "Dashboard", icon: "\u25C9" },
    { route: "explorer", label: "Explorer", icon: "\u229E" },
    { route: "deadletters", label: "Dead Letters", icon: "\u26A0" },
    { route: "nodes", label: "Nodes", icon: "\u2B21" },
    { route: "queues", label: "Queues", icon: "\u2261" },
    { route: "connections", label: "Connections", icon: "\u26C1" },
  ];

  function connNameForRoute(route: Route): string | null {
    if (route === "connections") return null;
    const match = $connections.find((c) => c.config.routes.includes(route));
    return match?.config.name ?? null;
  }
</script>

<aside class="w-56 h-screen flex flex-col bg-[var(--color-surface-raised)] border-r border-[var(--color-border)]">
  <div class="px-4 py-5 text-lg font-bold tracking-tight">
    Wolverine Monitor
  </div>

  <nav class="flex-1 px-2 space-y-1">
    {#each navItems as item}
      {@const connName = connNameForRoute(item.route)}
      <button
        class="w-full flex items-center gap-3 px-3 py-2 rounded-md text-sm transition-colors
          {$currentRoute === item.route
            ? 'bg-[var(--color-surface-overlay)] text-white'
            : 'text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-overlay)] hover:text-white'}"
        onclick={() => navigate(item.route)}
      >
        <span class="text-base">{item.icon}</span>
        <span class="flex-1 text-left">{item.label}</span>
        {#if connName}
          <span class="text-[10px] px-1.5 py-0.5 rounded bg-blue-500/20 text-blue-400 truncate max-w-20">{connName}</span>
        {:else if item.route !== "connections"}
          <span class="text-[10px] px-1.5 py-0.5 rounded bg-yellow-500/20 text-yellow-400">---</span>
        {/if}
      </button>
    {/each}
  </nav>

  <div class="px-4 py-3 text-xs text-[var(--color-text-secondary)] border-t border-[var(--color-border)]">
    v{version}
  </div>
</aside>
