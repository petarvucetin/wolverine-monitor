<script lang="ts">
  import type { ThroughputPoint } from "$lib/types";

  interface Props {
    data: ThroughputPoint[];
  }
  let { data }: Props = $props();
</script>

<div class="bg-[var(--color-surface-raised)] border border-[var(--color-border)] rounded-lg p-4">
  <div class="text-sm font-semibold mb-3">Throughput (last 30 min)</div>
  {#if data.length > 1}
    <div class="h-48 flex items-end gap-px">
      {#each data as point, i}
        {@const maxVal = Math.max(...data.map(d => d.incoming + d.outgoing), 1)}
        {@const total = point.incoming + point.outgoing}
        {@const height = (total / maxVal) * 100}
        <div class="flex-1 flex flex-col justify-end group relative" title="{point.incoming} in / {point.outgoing} out">
          {#if point.outgoing > 0}
            {@const outHeight = (point.outgoing / maxVal) * 100}
            <div class="bg-[var(--color-status-outgoing)] rounded-t-sm" style="height: {outHeight}%"></div>
          {/if}
          {#if point.incoming > 0}
            {@const inHeight = (point.incoming / maxVal) * 100}
            <div class="bg-[var(--color-status-incoming)] {point.outgoing === 0 ? 'rounded-t-sm' : ''}" style="height: {inHeight}%"></div>
          {/if}
        </div>
      {/each}
    </div>
    <div class="flex items-center gap-4 mt-2 text-xs text-[var(--color-text-secondary)]">
      <div class="flex items-center gap-1">
        <div class="w-2.5 h-2.5 rounded-sm bg-[var(--color-status-incoming)]"></div>
        <span>Incoming</span>
      </div>
      <div class="flex items-center gap-1">
        <div class="w-2.5 h-2.5 rounded-sm bg-[var(--color-status-outgoing)]"></div>
        <span>Outgoing</span>
      </div>
    </div>
  {:else}
    <div class="h-48 flex items-center justify-center text-sm text-[var(--color-text-secondary)]">
      Collecting throughput data...
    </div>
  {/if}
</div>
