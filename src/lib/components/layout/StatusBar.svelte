<script lang="ts">
  import { statusBar } from "$lib/stores/statusBar";
  import { connections, activeConnection } from "$lib/stores/connections";

  function statusColor(s: string): string {
    switch (s) {
      case "ready": return "var(--color-success, #22c55e)";
      case "loading": return "var(--color-info, #3b82f6)";
      case "error": return "var(--color-error, #ef4444)";
      default: return "var(--color-text-secondary)";
    }
  }
</script>

<div class="flex items-center gap-4 px-4 py-1.5 text-xs border-t border-[var(--color-border)] bg-[var(--color-surface-raised)] text-[var(--color-text-secondary)]">
  <div class="flex items-center gap-1.5">
    <span class="inline-block w-2 h-2 rounded-full" style="background-color: {statusColor($statusBar.status)}"></span>
    <span>{$statusBar.message}</span>
  </div>

  <div class="flex-1"></div>

  {#if $activeConnection}
    <span>{$activeConnection.config.name}</span>
    <span class="opacity-50">|</span>
    <span>{$activeConnection.config.schema}</span>
  {/if}

  <span class="opacity-50">|</span>
  <span>{$connections.length} connection{$connections.length !== 1 ? 's' : ''}</span>
</div>
