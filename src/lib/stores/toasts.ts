import { writable } from "svelte/store";

export interface Toast {
  id: number;
  message: string;
  type: "info" | "error" | "success" | "warning";
}

let nextId = 0;

function createToastStore() {
  const { subscribe, update } = writable<Toast[]>([]);

  return {
    subscribe,
    add(message: string, type: Toast["type"] = "info") {
      const id = nextId++;
      update((toasts) => [...toasts, { id, message, type }]);
      setTimeout(() => {
        update((toasts) => toasts.filter((t) => t.id !== id));
      }, 5000);
    },
    dismiss(id: number) {
      update((toasts) => toasts.filter((t) => t.id !== id));
    },
  };
}

export const toasts = createToastStore();
