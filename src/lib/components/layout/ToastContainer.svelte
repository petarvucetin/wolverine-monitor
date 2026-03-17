<script lang="ts">
  import { toasts } from "$lib/stores/toasts";

  const colorMap: Record<string, string> = {
    info: "border-blue-500",
    error: "border-red-500",
    success: "border-green-500",
    warning: "border-orange-500",
  };
</script>

<div class="fixed top-4 right-4 z-50 flex flex-col gap-2 max-w-sm">
  {#each $toasts as toast (toast.id)}
    <div
      class="bg-[var(--color-surface-raised)] border-l-4 {colorMap[toast.type]} px-4 py-3 rounded shadow-lg text-sm"
      role="alert"
    >
      <div class="flex justify-between items-start gap-2">
        <span>{toast.message}</span>
        <button
          class="text-[var(--color-text-secondary)] hover:text-white"
          onclick={() => toasts.dismiss(toast.id)}
        >
          ✕
        </button>
      </div>
    </div>
  {/each}
</div>
