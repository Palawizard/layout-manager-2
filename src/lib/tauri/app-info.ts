import { invokeCommand } from "./client";

export interface AppInfo {
  name: string;
  version: string;
  platform: "windows";
}

export function getAppInfo() {
  return invokeCommand<AppInfo>("get_app_info");
}
