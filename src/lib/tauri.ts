import { invoke } from "@tauri-apps/api/core";
import type { PanelState } from "./types";

export function loadPanelState(): Promise<PanelState> {
  return invoke<PanelState>("load_panel_state");
}

export function switchAccount(accountKey: string): Promise<PanelState> {
  return invoke<PanelState>("switch_account", { accountKey });
}

export function setAutoSwitch(enabled: boolean): Promise<PanelState> {
  return invoke<PanelState>("set_auto_switch", { enabled });
}

export function setUsageApi(enabled: boolean): Promise<PanelState> {
  return invoke<PanelState>("set_usage_api", { enabled });
}

export function quitApp(): Promise<void> {
  return invoke<void>("quit_app");
}
