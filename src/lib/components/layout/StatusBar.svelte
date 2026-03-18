<script lang="ts">
  import { statusBar } from "$lib/stores/statusBar";
  import { connections } from "$lib/stores/connections";
  import { getVersion } from "@tauri-apps/api/app";

  let appVersion = $state("");

  $effect(() => {
    getVersion().then((v) => { appVersion = v; });
  });

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

  <span>{$connections.length} connection{$connections.length !== 1 ? 's' : ''}</span>

  {#if appVersion}
    <span class="opacity-50">|</span>
    <span>v{appVersion}</span>
  {/if}
</div>
