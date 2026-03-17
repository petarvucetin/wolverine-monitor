<script lang="ts">
  import { currentRoute, navigate } from "$lib/stores/router";
  import type { Route } from "$lib/types";

  const navItems: { route: Route; label: string; icon: string }[] = [
    { route: "dashboard", label: "Dashboard", icon: "◉" },
    { route: "explorer", label: "Explorer", icon: "⊞" },
    { route: "deadletters", label: "Dead Letters", icon: "⚠" },
    { route: "nodes", label: "Nodes", icon: "⬡" },
    { route: "connections", label: "Connections", icon: "⛁" },
  ];
</script>

<aside class="w-56 h-screen flex flex-col bg-[var(--color-surface-raised)] border-r border-[var(--color-border)]">
  <div class="px-4 py-5 text-lg font-bold tracking-tight">
    Wolverine Monitor
  </div>

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
