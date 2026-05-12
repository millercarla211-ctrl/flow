"use client";

import {
  Box,
  FileText,
  FileType,
  Mail,
  Mic,
  MoreHorizontal,
  Radio,
  Send,
  Video,
} from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { useEffect, useRef, useState } from "react";
import { Button } from "@/liquidglass/www/components/ui/button";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/liquidglass/www/components/ui/popover";
import { Textarea } from "@/liquidglass/www/components/ui/textarea";
import { cn } from "@/liquidglass/www/lib/utils";
import { AIModeSwitcher } from "./ai-mode-switcher";
import { AITargetSwitcher } from "./ai-target-switcher";
import { ImageControls, MediaControls } from "./media-controls";
import { ModelPicker } from "./model-picker";

interface MediaType {
  id: string;
  label: string;
  icon: React.ElementType;
}

interface Message {
  role: string;
  content: string;
  id: string;
}

interface AIInputBarProps {
  messages: Message[];
  onMessagesChange: (messages: Message[]) => void;
  isLoading: boolean;
  onLoadingChange: (loading: boolean) => void;
}

const MEDIA_TYPES: MediaType[] = [
  { id: "text", label: "Text", icon: FileText },
  { id: "image", label: "Image", icon: FileType },
  { id: "video", label: "Video", icon: Video },
  { id: "audio", label: "Audio", icon: Mic },
  { id: "email", label: "Email", icon: Mail },
  { id: "live", label: "Live", icon: Radio },
  { id: "3d", label: "3D", icon: Box },
];

export function AIInputBar({ messages, onMessagesChange, isLoading, onLoadingChange }: AIInputBarProps) {
  const [selectedMedia, setSelectedMedia] = useState<string>("text");
  const [selectedProvider, setSelectedProvider] = useState<string>("gemini");
  const [selectedModel, setSelectedModel] =
    useState<string>("gemini-3.1-flash-lite-preview");
  const [input, setInput] = useState("");
  const [isFocused, setIsFocused] = useState(false);
  const [showMoreMedia, setShowMoreMedia] = useState(false);
  const [visibleMediaCount, setVisibleMediaCount] = useState(7);
  const [showMediaLabels, setShowMediaLabels] = useState(true);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  // Calculate visible media items and label visibility based on container width
  useEffect(() => {
    const updateVisibleMedia = () => {
      if (!containerRef.current) return;
      const width = containerRef.current.offsetWidth;

      if (width < 600) {
        setVisibleMediaCount(1);
        setShowMediaLabels(false);
      } else if (width < 750) {
        setVisibleMediaCount(3);
        setShowMediaLabels(false);
      } else if (width < 900) {
        setVisibleMediaCount(5);
        setShowMediaLabels(false);
      } else {
        setVisibleMediaCount(7);
        setShowMediaLabels(true);
      }
    };

    updateVisibleMedia();
    const ro = new ResizeObserver(updateVisibleMedia);
    if (containerRef.current) {
      ro.observe(containerRef.current);
    }
    return () => ro.disconnect();
  }, []);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSubmit(e);
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!input.trim() || isLoading) return;

    const userMessage = {
      role: "user",
      content: input,
      id: Date.now().toString(),
    };

    onMessagesChange([...messages, userMessage]);
    setInput("");
    onLoadingChange(true);

    try {
      const response = await fetch("/api/chat", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          messages: [...messages, userMessage],
          providerId: selectedProvider,
          modelId: selectedModel,
        }),
      });

      if (!response.ok) throw new Error("Failed to fetch");

      const reader = response.body?.getReader();
      const decoder = new TextDecoder();
      let assistantMessage = "";

      const assistantId = Date.now().toString();
      onMessagesChange([
        ...messages,
        userMessage,
        { role: "assistant", content: "", id: assistantId },
      ]);

      while (reader) {
        const { done, value } = await reader.read();
        if (done) break;

        const chunk = decoder.decode(value);
        const lines = chunk.split("\n");

        for (const line of lines) {
          if (line.trim() === "" || line === "data: [DONE]") continue;

          if (line.startsWith("data: ")) {
            try {
              const jsonStr = line.slice(6);
              const parsed = JSON.parse(jsonStr);

              if (parsed.type === "text-delta" && parsed.delta) {
                assistantMessage += parsed.delta;
                onMessagesChange([
                  ...messages,
                  userMessage,
                  { role: "assistant", content: assistantMessage, id: assistantId },
                ]);
              }
            } catch {
              console.log("Failed to parse SSE data:", line);
            }
          }
        }
      }
    } catch (error) {
      console.error("Chat error:", error);
    } finally {
      onLoadingChange(false);
    }
  };

  const visibleMedia = MEDIA_TYPES.slice(0, visibleMediaCount);
  const hiddenMedia = MEDIA_TYPES.slice(visibleMediaCount);

  return (
    <motion.div
      ref={containerRef}
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.3 }}
      className={cn(
        "bg-background/10 backdrop-blur-xl border-white/10 relative w-full rounded-2xl border shadow-lg transition-all duration-200",
        isFocused && "ring-2 ring-ring/30 shadow-2xl border-white/20 bg-background/20",
      )}
    >
      <form onSubmit={handleSubmit}>
        <div className="relative">
          <Textarea
            ref={textareaRef}
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={handleKeyDown}
            onFocus={() => setIsFocused(true)}
            onBlur={() => setIsFocused(false)}
            placeholder={
              selectedMedia === "image"
                ? "Describe the image you want to generate..."
                : selectedMedia === "video"
                  ? "Describe the video you want to create..."
                  : "Ask me anything or describe what you need help with..."
            }
            disabled={isLoading}
            className="text-foreground placeholder:text-muted-foreground min-h-px max-h-[120px] resize-none border-0 bg-transparent px-4 py-2 text-sm leading-tight focus-visible:ring-0"
          />
        </div>

        <div className="border-white/10 flex items-center justify-between gap-3 border-t bg-background/5 backdrop-blur-md px-4 py-2.5 rounded-b-2xl">
          <div className="flex items-center gap-2">
            <AITargetSwitcher />
            <AIModeSwitcher />

            {selectedMedia === "image" && <ImageControls />}
            {(selectedMedia === "video" ||
              selectedMedia === "audio" ||
              selectedMedia === "live" ||
              selectedMedia === "3d") && (
              <MediaControls mediaType={selectedMedia as any} />
            )}

            <ModelPicker
              selectedProvider={selectedProvider}
              selectedModel={selectedModel}
              onProviderChange={setSelectedProvider}
              onModelChange={setSelectedModel}
            />
          </div>

          <div className="flex items-center gap-2">
            <div className="flex items-center gap-1">
              <AnimatePresence mode="popLayout">
                {visibleMedia.map((media) => {
                  const Icon = media.icon;
                  const isSelected = selectedMedia === media.id;
                  return (
                    <motion.div
                      key={media.id}
                      layout
                      initial={{ opacity: 0, scale: 0.8 }}
                      animate={{ opacity: 1, scale: 1 }}
                      exit={{ opacity: 0, scale: 0.8 }}
                      transition={{ duration: 0.15 }}
                    >
                      <Button
                        type="button"
                        variant={isSelected ? "default" : "ghost"}
                        size="sm"
                        onClick={() => setSelectedMedia(media.id)}
                        className={cn(
                          "h-7 transition-all",
                          showMediaLabels ? "gap-1.5 px-2.5" : "w-7 px-0",
                        )}
                        title={media.label}
                      >
                        <Icon className="h-3.5 w-3.5 shrink-0" />
                        {showMediaLabels && (
                          <span className="text-xs">{media.label}</span>
                        )}
                      </Button>
                    </motion.div>
                  );
                })}
              </AnimatePresence>

              {hiddenMedia.length > 0 && (
                <Popover
                  open={showMoreMedia}
                  onOpenChange={setShowMoreMedia}
                >
                  <PopoverTrigger asChild>
                    <motion.div
                      initial={{ opacity: 0, scale: 0.8 }}
                      animate={{ opacity: 1, scale: 1 }}
                      transition={{ duration: 0.15 }}
                    >
                      <Button
                        type="button"
                        variant="ghost"
                        size="sm"
                        className="h-7 w-7 px-0"
                        title="More media types"
                      >
                        <MoreHorizontal className="h-3.5 w-3.5" />
                      </Button>
                    </motion.div>
                  </PopoverTrigger>
                  <PopoverContent
                    side="top"
                    align="end"
                    className="w-48 p-2"
                  >
                    <div className="space-y-1">
                      {hiddenMedia.map((media) => {
                        const Icon = media.icon;
                        const isSelected = selectedMedia === media.id;
                        return (
                          <Button
                            key={media.id}
                            type="button"
                            variant={isSelected ? "default" : "ghost"}
                            size="sm"
                            onClick={() => {
                              setSelectedMedia(media.id);
                              setShowMoreMedia(false);
                            }}
                            className="w-full justify-start gap-2 h-8"
                          >
                            <Icon className="h-3.5 w-3.5" />
                            <span className="text-sm">{media.label}</span>
                          </Button>
                        );
                      })}
                    </div>
                  </PopoverContent>
                </Popover>
              )}
            </div>

            <motion.div
              whileHover={{ scale: 1.02 }}
              whileTap={{ scale: 0.98 }}
            >
              <Button
                type="submit"
                disabled={!input.trim() || isLoading}
                size="sm"
                className="h-8 gap-2 px-3 bg-primary hover:bg-primary/90 disabled:opacity-50 text-primary-foreground"
              >
                <Send className="h-3.5 w-3.5" />
                <span className="text-sm font-medium">
                  {isLoading ? "Sending..." : "Send"}
                </span>
              </Button>
            </motion.div>
          </div>
        </div>
      </form>
    </motion.div>
  );
}
