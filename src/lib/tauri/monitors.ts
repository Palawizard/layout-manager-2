import { invokeCommand } from "./client";

export interface WorkArea {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface Monitor {
  id: string;
  name: string;
  workArea: WorkArea;
  scaleFactor: number;
  isPrimary: boolean;
}

export function listMonitors() {
  return invokeCommand<Monitor[]>("list_monitors");
}
