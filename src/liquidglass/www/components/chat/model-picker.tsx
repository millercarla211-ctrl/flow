"use client";

import { Bot, ChevronDown, Cloud, Laptop } from "lucide-react";
import { motion } from "motion/react";
import type { ComponentType } from "react";
import { Button } from "@/liquidglass/www/components/ui/button";
import { Popover, PopoverContent, PopoverTrigger } from "@/liquidglass/www/components/ui/popover";
import { ScrollArea } from "@/liquidglass/www/components/ui/scroll-area";
import { cn } from "@/liquidglass/www/lib/utils";

type ProviderId = "local" | "flow" | "cloud";

const providers: Record<
  ProviderId,
  {
    id: ProviderId;
    name: string;
    icon: ComponentType<{ className?: string }>;
    defaultModel: string;
    models: Array<{ id: string; name: string; description: string }>;
  }
> = {
  local: {
    id: "local",
    name: "Local",
    icon: Laptop,
    defaultModel: "parakeet-unified-en-0.6b-int8",
    models: [
      {
        id: "parakeet-unified-en-0.6b-int8",
        name: "Parakeet Unified",
        description: "Fast local speech model used by Friday.",
      },
      {
        id: "qwen3-0.6b",
        name: "Qwen3 0.6B",
        description: "Low-latency local cleanup helper.",
      },
    ],
  },
  flow: {
    id: "flow",
    name: "Friday",
    icon: Bot,
    defaultModel: "qwen35-4b-revised-q4km",
    models: [
      {
        id: "qwen35-4b-revised-q4km",
        name: "Qwen 3.5 4B",
        description: "Daily local assistant brain.",
      },
      {
        id: "xlam2-3b-fc-r-q4km",
        name: "xLAM 2 3B FC",
        description: "Local tool-routing model.",
      },
    ],
  },
  cloud: {
    id: "cloud",
    name: "Optional Remote",
    icon: Cloud,
    defaultModel: "bring-your-own-provider",
    models: [
      {
        id: "bring-your-own-provider",
        name: "Configured provider",
        description: "Remote models only when the user opts in.",
      },
    ],
  },
};

interface ModelPickerProps {
  selectedProvider: string;
  selectedModel: string;
  onProviderChange: (providerId: string) => void;
  onModelChange: (modelId: string) => void;
}

export function ModelPicker({
  selectedProvider,
  selectedModel,
  onProviderChange,
  onModelChange,
}: ModelPickerProps) {
  const currentProvider = providers[selectedProvider as ProviderId];
  const currentModel = currentProvider?.models.find((m) => m.id === selectedModel);

  const handleProviderChange = (providerId: string) => {
    onProviderChange(providerId);
    // Auto-select the default model for the new provider
    const newProvider = providers[providerId as ProviderId];
    if (newProvider) {
      onModelChange(newProvider.defaultModel);
    }
  };

  return (
    <Popover>
      <PopoverTrigger asChild>
        <Button variant="outline" size="sm" className="h-8 gap-2 px-3">
          {currentProvider && <currentProvider.icon className="h-3.5 w-3.5" />}
          <span className="text-xs">
            {currentModel?.name || currentProvider?.name || "Select Model"}
          </span>
          <ChevronDown className="h-3.5 w-3.5 opacity-50" />
        </Button>
      </PopoverTrigger>
      <PopoverContent side="top" align="start" className="w-96 p-0">
        <div className="border-border border-b p-3">
          <h3 className="text-foreground text-sm font-semibold">Select AI Model</h3>
          <p className="text-muted-foreground text-xs">Choose provider and specific model</p>
        </div>
        <ScrollArea className="h-[500px]">
          <div className="space-y-3 p-3">
            {Object.values(providers).map((provider) => {
              const isProviderSelected = selectedProvider === provider.id;
              return (
                <div key={provider.id} className="space-y-2">
                  <div className="flex items-center gap-2 px-2">
                    <provider.icon className="h-4 w-4" />
                    <span className="text-xs font-medium text-muted-foreground uppercase tracking-wide">
                      {provider.name}
                    </span>
                    <div className="flex-1 h-px bg-border" />
                  </div>
                  <div className="space-y-1">
                    {provider.models.map((model) => {
                      const isSelected = isProviderSelected && selectedModel === model.id;
                      return (
                        <motion.button
                          key={model.id}
                          onClick={() => {
                            handleProviderChange(provider.id);
                            onModelChange(model.id);
                          }}
                          whileHover={{ scale: 1.01 }}
                          whileTap={{ scale: 0.99 }}
                          className={cn(
                            "flex w-full items-center gap-3 rounded-lg px-3 py-2.5 text-left transition-all",
                            isSelected
                              ? "bg-primary text-primary-foreground"
                              : "hover:bg-accent hover:text-accent-foreground",
                          )}
                        >
                          <div className="flex-1">
                            <div className="text-sm font-medium">{model.name}</div>
                            <div
                              className={cn(
                                "text-xs",
                                isSelected ? "text-primary-foreground/80" : "text-muted-foreground",
                              )}
                            >
                              {model.description}
                            </div>
                          </div>
                          {isSelected && <div className="h-2 w-2 rounded-full bg-current" />}
                        </motion.button>
                      );
                    })}
                  </div>
                </div>
              );
            })}
          </div>
        </ScrollArea>
      </PopoverContent>
    </Popover>
  );
}
