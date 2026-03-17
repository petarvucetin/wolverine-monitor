<script lang="ts">
  import type { IncomingEnvelope, OutgoingEnvelope, DeadLetter } from "$lib/types";
  import { decodeBody } from "$lib/format";

  type AnyEnvelope = IncomingEnvelope | OutgoingEnvelope | DeadLetter;

  interface Props {
    item: AnyEnvelope;
    onClose: () => void;
  }

  let { item, onClose }: Props = $props();

  let decoded = $derived(decodeBody(item.body));
  let copied = $state(false);

  async function copyBase64() {
    if (decoded.type === "hex") {
      await navigator.clipboard.writeText(decoded.base64);
      copied = true;
      setTimeout(() => { copied = false; }, 2000);
    }
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="fixed inset-0 z-40" onclick={onClose}>
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="fixed top-0 right-0 h-full w-[480px] bg-[var(--color-surface-raised)] border-l border-[var(--color-border)] shadow-xl z-50 overflow-y-auto"
    onclick={(e) => e.stopPropagation()}
  >
    <div class="flex items-center justify-between px-4 py-3 border-b border-[var(--color-border)]">
      <h3 class="text-sm font-semibold">Message Detail</h3>
      <button
        onclick={onClose}
        class="text-[var(--color-text-secondary)] hover:text-white text-lg leading-none"
      >
        &times;
      </button>
    </div>

    <div class="p-4 space-y-4">
      <div>
        <div class="text-xs text-[var(--color-text-secondary)] uppercase mb-1">ID</div>
        <div class="font-mono text-xs break-all">{item.id}</div>
      </div>

      <div>
        <div class="text-xs text-[var(--color-text-secondary)] uppercase mb-1">Message Type</div>
        <div class="text-sm break-all">{item.message_type}</div>
      </div>

      {#if "status" in item}
        <div>
          <div class="text-xs text-[var(--color-text-secondary)] uppercase mb-1">Status</div>
          <div class="text-sm">{item.status}</div>
        </div>
      {/if}

      {#if "destination" in item}
        <div>
          <div class="text-xs text-[var(--color-text-secondary)] uppercase mb-1">Destination</div>
          <div class="text-sm break-all">{item.destination}</div>
        </div>
      {/if}

      {#if "exception_type" in item && item.exception_type}
        <div>
          <div class="text-xs text-[var(--color-text-secondary)] uppercase mb-1">Exception Type</div>
          <div class="text-sm text-red-400">{item.exception_type}</div>
        </div>
      {/if}

      {#if "exception_message" in item && item.exception_message}
        <div>
          <div class="text-xs text-[var(--color-text-secondary)] uppercase mb-1">Exception Message</div>
          <div class="text-sm text-red-400 break-all">{item.exception_message}</div>
        </div>
      {/if}

      {#if "attempts" in item}
        <div>
          <div class="text-xs text-[var(--color-text-secondary)] uppercase mb-1">Attempts</div>
          <div class="text-sm">{item.attempts}</div>
        </div>
      {/if}

      <div>
        <div class="text-xs text-[var(--color-text-secondary)] uppercase mb-1">Body</div>
        {#if decoded.type === "json"}
          <pre class="p-3 rounded bg-[var(--color-surface)] border border-[var(--color-border)] text-xs overflow-x-auto max-h-96 whitespace-pre-wrap">{decoded.content}</pre>
        {:else}
          <div class="space-y-2">
            <pre class="p-3 rounded bg-[var(--color-surface)] border border-[var(--color-border)] text-xs overflow-x-auto max-h-96 whitespace-pre-wrap font-mono">{decoded.content}</pre>
            <button
              onclick={copyBase64}
              class="px-3 py-1 text-xs rounded border border-[var(--color-border)] hover:bg-[var(--color-surface-overlay)] transition-colors"
            >
              {copied ? "Copied!" : "Copy Base64"}
            </button>
          </div>
        {/if}
      </div>
    </div>
  </div>
</div>
