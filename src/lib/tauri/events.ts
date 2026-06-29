import { listen, type Event, type UnlistenFn } from "@tauri-apps/api/event";

export function subscribeToEvent<TPayload>(
  eventName: string,
  handler: (event: Event<TPayload>) => void,
): Promise<UnlistenFn> {
  return listen<TPayload>(eventName, handler);
}
