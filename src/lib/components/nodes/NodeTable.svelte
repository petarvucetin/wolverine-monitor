<script lang="ts">
  import type { WolverineNode } from "$lib/types";
  import { formatRelativeTime } from "$lib/format";
  import HealthIndicator from "./HealthIndicator.svelte";

  interface Props {
    nodes: WolverineNode[];
  }

  let { nodes }: Props = $props();
</script>

<div class="rounded-lg bg-[var(--color-surface-raised)] border border-[var(--color-border)] overflow-hidden">
  {#if nodes.length === 0}
    <div class="px-4 py-8 text-center text-sm text-[var(--color-text-secondary)]">
      No nodes found.
    </div>
  {:else}
    <table class="w-full text-sm">
      <thead>
        <tr class="border-b border-[var(--color-border)] text-left text-xs uppercase text-[var(--color-text-secondary)]">
          <th class="px-4 py-3">Node</th>
          <th class="px-4 py-3">Health</th>
          <th class="px-4 py-3">URI</th>
          <th class="px-4 py-3">Version</th>
          <th class="px-4 py-3">Started</th>
          <th class="px-4 py-3">Last Check</th>
        </tr>
      </thead>
      <tbody>
        {#each nodes as node (node.id)}
          <tr class="border-b border-[var(--color-border)] last:border-b-0 hover:bg-[var(--color-surface-overlay)] transition-colors">
            <td class="px-4 py-2">
              <div class="font-medium">#{node.node_number}</div>
              {#if node.description}
                <div class="text-xs text-[var(--color-text-secondary)]">{node.description}</div>
              {/if}
            </td>
            <td class="px-4 py-2">
              <HealthIndicator healthCheck={node.health_check} />
            </td>
            <td class="px-4 py-2 font-mono text-xs">
              {node.uri ?? "\u2014"}
            </td>
            <td class="px-4 py-2">
              {node.version ?? "\u2014"}
            </td>
            <td class="px-4 py-2 text-[var(--color-text-secondary)]">
              {formatRelativeTime(node.started)}
            </td>
            <td class="px-4 py-2 text-[var(--color-text-secondary)]">
              {formatRelativeTime(node.health_check)}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</div>
