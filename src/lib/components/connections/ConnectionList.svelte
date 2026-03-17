<script lang="ts">
  import { connections, activeConnectionId, deleteConnection } from "$lib/stores/connections";
  import ConnectionStatus from "./ConnectionStatus.svelte";

  function activate(id: string) {
    activeConnectionId.set(id);
  }
</script>

{#if $connections.length === 0}
  <p class="text-sm text-[var(--color-text-secondary)] p-4">No connections configured.</p>
{:else}
  <div class="space-y-2">
    {#each $connections as conn (conn.config.id)}
      <div
        class="flex items-center justify-between p-3 rounded-lg border transition-colors cursor-pointer
          {$activeConnectionId === conn.config.id
            ? 'border-blue-500 bg-[var(--color-surface-overlay)]'
            : 'border-[var(--color-border)] bg-[var(--color-surface-raised)] hover:bg-[var(--color-surface-overlay)]'}"
        onclick={() => activate(conn.config.id)}
        onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); activate(conn.config.id); } }}
        role="button"
        tabindex="0"
      >
        <div>
          <div class="flex items-center gap-2">
            <span class="text-sm font-medium">{conn.config.name}</span>
            {#each conn.config.routes as route}
              <span class="text-[10px] px-1.5 py-0.5 rounded bg-blue-500/20 text-blue-400 font-medium capitalize">{route}</span>
            {/each}
          </div>
          <div class="text-xs text-[var(--color-text-secondary)]">
            {conn.config.host}:{conn.config.port}/{conn.config.database} ({conn.config.schema})
          </div>
        </div>
        <div class="flex items-center gap-3">
          <ConnectionStatus status={conn.status} />
          <button
            onclick={(e) => { e.stopPropagation(); deleteConnection(conn.config.id); }}
            class="text-xs text-red-400 hover:text-red-300"
          >
            Remove
          </button>
        </div>
      </div>
    {/each}
  </div>
{/if}
