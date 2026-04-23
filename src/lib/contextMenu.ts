import { writable } from "svelte/store";
import type { TileModel } from "./stores";

export interface ContextMenuState {
  tile: TileModel;
  x: number;
  y: number;
}

export const contextMenu = writable<ContextMenuState | null>(null);

export function openMenu(tile: TileModel, event: MouseEvent) {
  event.preventDefault();
  contextMenu.set({ tile, x: event.clientX, y: event.clientY });
}

export function closeMenu() {
  contextMenu.set(null);
}
