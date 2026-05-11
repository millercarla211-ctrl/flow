import { invoke } from "@tauri-apps/api/core";
import type { Snippet } from "../../types";

export async function listSnippets(): Promise<Snippet[]> {
  return invoke<Snippet[]>("list_snippets");
}

export async function createSnippet(input: {
  trigger: string;
  expansion: string;
}): Promise<Snippet> {
  return invoke<Snippet>("create_snippet", input);
}

export async function updateSnippet(input: {
  id: string;
  trigger: string;
  expansion: string;
}): Promise<Snippet | null> {
  return invoke<Snippet | null>("update_snippet", input);
}

export async function deleteSnippet(id: string): Promise<boolean> {
  return invoke<boolean>("delete_snippet", { id });
}
