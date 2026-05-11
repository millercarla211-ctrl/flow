import { invoke } from "@tauri-apps/api/core";
import type { StoredSettings, AppInfo, DeviceInfo, AutoPasteStatus } from "../../types";

export async function getSettings(): Promise<StoredSettings> {
  return invoke<StoredSettings>("get_settings");
}

export async function getAppInfo(): Promise<AppInfo> {
  return invoke<AppInfo>("get_app_info");
}

export async function listInputDevices(): Promise<DeviceInfo[]> {
  return invoke<DeviceInfo[]>("list_input_devices");
}

export async function getAutoPasteStatus(): Promise<AutoPasteStatus> {
  return invoke<AutoPasteStatus>("get_auto_paste_status");
}
