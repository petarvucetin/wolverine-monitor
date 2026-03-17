<script lang="ts">
  import { recentMessages } from "$lib/stores/dashboard";
  import { shortenMessageType } from "$lib/format";

  function tableIcon(table: string): string {
    switch (table) {
      case "incoming": return "\u2192";
      case "outgoing": return "\u2190";
      case "dead_letter": return "\u2717";
      default: return "?";
    }
  }

  function tableColor(table: string): string {
    switch (table) {
      case "incoming": return "var(--color-success, #22c55e)";
      case "outgoing": return "var(--color-info, #3b82f6)";
      case "dead_letter": return "var(--color-error, #ef4444)";
      default: return "var(--color-text-secondary)";
    }
  }

  function relativeTime(ts: number): string {
    const diff = Math.floor((Date.now() - ts) / 1000);
    if (diff < 5) return "just now";
    if (diff < 60) return `${diff}s ago`;
    const minutes = Math.floor(diff / 60);
    if (minutes < 60) return `${minutes}m ago`;
    return `${Math.floor(minutes / 60)}h ago`;
  }
</script>

<div class="rounded-lg bg-[var(--color-surface-raised)] border border-[var(--color-border)] overflow-hidden">
  <div class="px-4 py-3 border-b border-[var(--color-border)]">
    <h3 class="text-sm font-semibold">Live Feed</h3>
  </div>
  <div class="overflow-y-auto max-h-96">
    {#if $recentMessages.length === 0}
      <div class="px-4 py-8 text-center text-sm text-[var(--color-text-secondary)]">
        No messages yet. Waiting for activity...
      </div>
    {:else}
      {#each $recentMessages.slice(0, 50) as msg (msg.id + msg.timestamp)}
        <div class="flex items-center gap-3 px-4 py-2 text-sm border-b border-[var(--color-border)] last:border-b-0 hover:bg-[var(--color-surface-overlay)]">
          <span class="text-base w-5 text-center" style="color: {tableColor(msg.table)}">
            {tableIcon(msg.table)}
          </span>
          <span class="flex-1 truncate font-mono text-xs">
            {shortenMessageType(msg.message_type)}
          </span>
          <span class="text-xs text-[var(--color-text-secondary)] capitalize">
            {msg.table.replace("_", " ")}
          </span>
          <span class="text-xs text-[var(--color-text-secondary)] w-16 text-right">
            {relativeTime(msg.timestamp)}
          </span>
        </div>
      {/each}
    {/if}
  </div>
</div>
