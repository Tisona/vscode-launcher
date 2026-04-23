import { writable } from "svelte/store";

export interface Toast {
  id: number;
  message: string;
  kind: "error" | "info";
}

export const toasts = writable<Toast[]>([]);
let nextId = 1;

export function pushToast(message: string, kind: Toast["kind"] = "error") {
  const id = nextId++;
  toasts.update((list) => [...list, { id, message, kind }]);
  setTimeout(() => {
    toasts.update((list) => list.filter((t) => t.id !== id));
  }, 5000);
}
