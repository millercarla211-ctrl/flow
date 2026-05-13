import type { ChatTransport, UIMessage, UIMessageChunk } from "ai";

import {
  createLocalAssistantDraft,
  type FridayChatContext,
  getTextFromUiMessage,
  streamLocalText,
} from "./local-stream";
import { resolveFridayModel } from "./model-routing";
import { tryRunTauriLocalChat } from "./tauri-local-chat";

type ModelKeyResolver = () => string;
type ContextResolver = () => FridayChatContext | undefined;

export class LocalFridayChatTransport implements ChatTransport<UIMessage> {
  constructor(
    private readonly getModelKey: ModelKeyResolver,
    private readonly getContext?: ContextResolver,
  ) {}

  async sendMessages({
    messages,
    abortSignal,
  }: Parameters<ChatTransport<UIMessage>["sendMessages"]>[0]): Promise<
    ReadableStream<UIMessageChunk>
  > {
    const model = resolveFridayModel(this.getModelKey());
    const prompt = getTextFromUiMessage(messages[messages.length - 1]);
    const context = this.getContext?.();
    const tauriResult = await tryRunTauriLocalChat({ prompt, model, context });
    const response = tauriResult?.text ?? createLocalAssistantDraft(prompt, model, context);
    const textId = `friday-local-${Date.now().toString(36)}`;

    return new ReadableStream<UIMessageChunk>({
      async start(controller) {
        controller.enqueue({ type: "start" });
        controller.enqueue({ type: "start-step" });
        controller.enqueue({ type: "text-start", id: textId });

        for await (const delta of streamLocalText(response, abortSignal)) {
          controller.enqueue({ type: "text-delta", id: textId, delta });
        }

        controller.enqueue({ type: "text-end", id: textId });
        controller.enqueue({ type: "finish-step" });
        controller.enqueue({
          type: "finish",
          finishReason: abortSignal?.aborted ? "other" : "stop",
        });
        controller.close();
      },
      cancel() {
        return undefined;
      },
    });
  }

  async reconnectToStream(): Promise<ReadableStream<UIMessageChunk> | null> {
    return null;
  }
}
