<script lang="ts">
  import type { QueueInfo } from "$lib/types";

  interface Props {
    queues: QueueInfo[];
    selectedQueue: string | null;
    onSelect: (queue: QueueInfo) => void;
  }

  let { queues, selectedQueue, onSelect }: Props = $props();

  function depthColor(count: number): string {
    if (count === 0) return "var(--color-success, #22c55e)";
    if (count < 100) return "var(--color-info, #3b82f6)";
    if (count < 1000) return "#eab308";
    return "var(--color-error, #ef4444)";
  }
</script>

{#if queues.length === 0}
  <p class="text-sm text-[var(--color-text-secondary)] py-4">No queues found.</p>
{:else}
  <div class="overflow-x-auto">
    <table class="w-full text-sm">
      <thead>
        <tr class="text-left text-xs text-[var(--color-text-secondary)] border-b border-[var(--color-border)]">
          <th class="px-3 py-2">Queue</th>
          <th class="px-3 py-2 text-right">Messages</th>
          <th class="px-3 py-2 text-right">Scheduled</th>
          <th class="px-3 py-2 text-right">Total</th>
        </tr>
      </thead>
      <tbody>
        {#each queues as queue (queue.table_name)}
          <tr
            class="border-b border-[var(--color-border)] cursor-pointer transition-colors
              {selectedQueue === queue.name
                ? 'bg-[var(--color-surface-overlay)]'
                : 'hover:bg-[var(--color-surface-overlay)]'}"
            onclick={() => onSelect(queue)}
          >
            <td class="px-3 py-2 font-mono text-xs">{queue.name}</td>
            <td class="px-3 py-2 text-right">
              <span style="color: {depthColor(queue.count)}">{queue.count}</span>
            </td>
            <td class="px-3 py-2 text-right">
              {#if queue.has_scheduled_table}
                <span style="color: {depthColor(queue.scheduled_count)}">{queue.scheduled_count}</span>
              {:else}
                <span class="text-[var(--color-text-secondary)]">-</span>
              {/if}
            </td>
            <td class="px-3 py-2 text-right font-medium">
              <span style="color: {depthColor(queue.count + queue.scheduled_count)}">{queue.count + queue.scheduled_count}</span>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
{/if}
