import { invoke } from "@tauri-apps/api/core";
import type { LocalModelRuntimeStatus, ModelInfo, ModelStatus } from "../../types";

export async function listModels(): Promise<ModelInfo[]> {
  return invoke<ModelInfo[]>("list_models");
}

export async function listTtsModels(): Promise<ModelInfo[]> {
  return invoke<ModelInfo[]>("list_tts_models");
}

export async function checkModelStatus(model: string): Promise<ModelStatus> {
  return invoke<ModelStatus>("check_model_status", { model });
}

export async function checkTtsModelStatus(model: string): Promise<ModelStatus> {
  return invoke<ModelStatus>("check_tts_model_status", { model });
}

export async function getLocalModelRuntimeStatus(): Promise<LocalModelRuntimeStatus> {
  return invoke<LocalModelRuntimeStatus>("get_local_model_runtime_status");
}
