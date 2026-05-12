"use client";

import { motion } from "motion/react";
import {
  Code,
  Copy,
  Image,
  Mic,
  Pause,
  Play,
  Settings,
  Star,
  ThumbsDown,
  ThumbsUp,
  Video,
  Zap,
} from "lucide-react";
import { useEffect, useRef, useState } from "react";
import ReactMarkdown from "react-markdown";
import { AIInputBar } from "@/liquidglass/www/components/chat/ai-input-bar";
import { HelloGlow } from "@/liquidglass/www/components/hello-glow";
import { Friday } from "@/liquidglass/www/components/friday";
import { ScrollArea } from "@/liquidglass/www/components/ui/scroll-area";
import { ThemeSwitcher } from "@/liquidglass/www/components/theme-switcher";
import { Card, CardContent, CardHeader, CardTitle } from "@/liquidglass/www/components/ui/card";
import { Button } from "@/liquidglass/www/components/ui/button";
import { Avatar, AvatarFallback } from "@/liquidglass/www/components/ui/avatar";
import { Slider } from "@/liquidglass/www/components/ui/slider";
import { Label } from "@/liquidglass/www/components/ui/label";
import { PixelCircle } from "@/liquidglass/www/components/PixelCircle";
import { EyesStage } from "@/liquidglass/www/components/eyes/eyes-stage";
import { AnimationControls } from "@/liquidglass/www/components/eyes/animation-controls";
import { ThemePicker } from "@/liquidglass/www/components/theme-picker";
import { SpidermanWavesMode } from "@/liquidglass/www/components/eyes/spiderman-waves";
import { cn } from "@/liquidglass/www/lib/utils";

const PALETTES = [
  ['#002b00', '#005e00', '#00a800', '#4dff4d', '#ffffff'],
  ['#330000', '#800000', '#e60000', '#ff6600', '#ffcc00', '#ffffff'],
  ['#1a0033', '#4d0099', '#9900cc', '#e600e6', '#ff99ff', '#ffffff'],
  ['#33001a', '#99004d', '#e60073', '#ff4d94', '#ffb3d1', '#ffffff'],
];

const featureCards = [
  {
    id: "featured",
    title: "Featured",
    description: "Our top picks including Gemini 3 Pro and Nano Banana Pro.",
    icon: Star,
    category: "featured",
  },
  {
    id: "code",
    title: "Code, Reasoning, and Chat",
    description:
      "Build chatbots, agents, and code with Gemini 3 Pro and Gemini 3 Flash.",
    icon: Code,
    category: "development",
  },
  {
    id: "image",
    title: "Image Generation",
    description: "Create and edit images with Nano Banana and Imagen.",
    icon: Image,
    category: "creative",
  },
  {
    id: "video",
    title: "Video Generation",
    description:
      "Generate videos with Veo models, our state of the art video generation models.",
    icon: Video,
    category: "creative",
  },
  {
    id: "speech",
    title: "Text to Speech",
    description:
      "Convert text to speech with lifelike realism using Gemini TTS.",
    icon: Mic,
    category: "audio",
  },
  {
    id: "realtime",
    title: "Real-time",
    description: "Real-time voice and video with Gemini Live.",
    icon: Zap,
    category: "interactive",
  },
];

function ControlSlider({ 
  label, 
  value, 
  min, 
  max, 
  step, 
  onChange 
}: { 
  label: string;
  value: number;
  min: number;
  max: number;
  step: number;
  onChange: (val: number) => void;
}) {
  return (
    <div className="space-y-3">
      <div className="flex justify-between items-center">
        <Label className="text-sm font-medium">{label}</Label>
        <span className="text-muted-foreground text-xs font-mono">
          {value.toFixed(step % 1 !== 0 ? 1 : 0)}
        </span>
      </div>
      <Slider
        value={[value]}
        min={min}
        max={max}
        step={step}
        onValueChange={(vals) => onChange(vals[0])}
        className="w-full"
      />
    </div>
  );
}

interface Message {
  role: string;
  content: string;
  id: string;
}

interface WelcomeScreenProps {
  sidebarExpanded?: boolean;
}

export function WelcomeScreen({ sidebarExpanded = false }: WelcomeScreenProps) {
  const sidebarWidth = sidebarExpanded ? 360 : 56;
  
  const [isPlaying, setIsPlaying] = useState(true);
  const [speed, setSpeed] = useState(3);
  const [resolution, setResolution] = useState(12);
  const [circleSize, setCircleSize] = useState(96);
  const [overlap, setOverlap] = useState(32);
  const [noiseAmount, setNoiseAmount] = useState(0.15);
  const [messages, setMessages] = useState<Message[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [gunshotMode, setGunshotMode] = useState(false);
  const [spidermanMode, setSpidermanMode] = useState(false);
  
  const hasMessages = messages.length > 0;

  const messagesEndRef = useRef<HTMLDivElement>(null);

  // Auto-scroll to latest message
  useEffect(() => {
    if (hasMessages) {
      messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
    }
  }, [hasMessages]);

  return (
    <>
      {/* Spider-Man Waves Mode */}
      <SpidermanWavesMode active={spidermanMode} />
      
      {/* Friday Siri-style Border Animation */}
      <Friday sidebarWidth={sidebarWidth} />
      
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5 }}
        className="relative flex h-full w-full flex-col rounded-[20px] border border-border bg-background shadow-2xl"
      >
        {/* Scrollable Content Area */}
        <ScrollArea className="flex-1">
          <div className={cn(
            "h-full",
            hasMessages ? "flex flex-col" : "p-8 space-y-8"
          )}>
            {/* Welcome Content - Hidden when there are messages */}
            {!hasMessages && (
              <>
            {/* Eyes Animation */}
            <motion.div
              initial={{ opacity: 0, scale: 0.9 }}
              animate={{ opacity: 1, scale: 1 }}
              transition={{ duration: 0.6, ease: "easeOut" }}
              className="flex justify-center"
            >
              <EyesStage gunshotMode={gunshotMode}>
                {/* Animation Controls inside the provider */}
                <motion.div
                  initial={{ opacity: 0, y: 20 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ duration: 0.6, delay: 0.2, ease: "easeOut" }}
                  className="flex justify-center mt-8"
                >
                  <AnimationControls 
                    onGunshotModeChange={setGunshotMode}
                    onSpidermanModeChange={setSpidermanMode}
                  />
                </motion.div>
              </EyesStage>
            </motion.div>

            {/* Welcome Header */}
            <div className="text-center space-y-4">
              <h1 className="text-foreground text-2xl font-semibold">
                Start building with Gemini
              </h1>
              <ThemeSwitcher />
            </div>

            {/* Zen Theme Picker */}
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.6, delay: 0.3, ease: "easeOut" }}
              className="flex justify-center"
            >
              <ThemePicker />
            </motion.div>

            {/* PixelCircle Showcase */}
            <div className="flex flex-col items-center justify-center gap-8">
              <motion.div
                initial={{ opacity: 0, scale: 0.9 }}
                animate={{ opacity: 1, scale: 1 }}
                transition={{ duration: 0.8, ease: "easeOut" }}
                className="flex items-center justify-center"
              >
                {PALETTES.map((palette) => (
                  <motion.div
                    key={`palette-${palette[0]}`}
                    initial={{ x: -20, opacity: 0 }}
                    animate={{ x: 0, opacity: 1 }}
                    transition={{ duration: 0.5, delay: PALETTES.indexOf(palette) * 0.1 + 0.3 }}
                    style={{
                      marginLeft: PALETTES.indexOf(palette) === 0 ? 0 : `-${overlap}px`,
                      zIndex: PALETTES.length - PALETTES.indexOf(palette),
                    }}
                    className="relative rounded-full overflow-hidden border-[3px] border-background shadow-2xl"
                  >
                    <PixelCircle
                      palette={palette}
                      speed={speed}
                      resolution={resolution}
                      isPlaying={isPlaying}
                      timeOffset={PALETTES.indexOf(palette) * 1000}
                      size={circleSize}
                      noiseAmount={noiseAmount}
                    />
                  </motion.div>
                ))}
              </motion.div>

              {/* Controls */}
              <motion.div
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.6, delay: 0.6 }}
                className="w-full max-w-md"
              >
                <Card className="border-border bg-card/80 backdrop-blur-2xl shadow-2xl">
                  <CardHeader className="pb-4">
                    <div className="flex items-center justify-between">
                      <div className="space-y-1">
                        <CardTitle className="text-xl font-semibold flex items-center gap-2">
                          <Settings className="w-5 h-5 text-muted-foreground" />
                          Parameters
                        </CardTitle>
                        <p className="text-sm text-muted-foreground">
                          Adjust the pixelated gradient effect
                        </p>
                      </div>
                      <Button
                        onClick={() => setIsPlaying(!isPlaying)}
                        size="icon"
                        className="h-12 w-12 rounded-full shadow-lg"
                      >
                        {isPlaying ? (
                          <Pause className="w-5 h-5 fill-current" />
                        ) : (
                          <Play className="w-5 h-5 fill-current ml-0.5" />
                        )}
                      </Button>
                    </div>
                  </CardHeader>
                  <CardContent className="space-y-6">
                    <ControlSlider
                      label="Animation Speed"
                      value={speed}
                      min={0}
                      max={10}
                      step={0.1}
                      onChange={setSpeed}
                    />
                    <ControlSlider
                      label="Pixel Density"
                      value={resolution}
                      min={4}
                      max={48}
                      step={1}
                      onChange={setResolution}
                    />
                    <ControlSlider
                      label="Circle Size"
                      value={circleSize}
                      min={40}
                      max={200}
                      step={4}
                      onChange={setCircleSize}
                    />
                    <ControlSlider
                      label="Overlap Offset"
                      value={overlap}
                      min={0}
                      max={circleSize}
                      step={1}
                      onChange={setOverlap}
                    />
                    <ControlSlider
                      label="Static Noise"
                      value={noiseAmount}
                      min={0}
                      max={0.5}
                      step={0.01}
                      onChange={setNoiseAmount}
                    />
                  </CardContent>
                </Card>
              </motion.div>
            </div>

            {/* HelloGlow Showcase Rectangle */}
            <div className="flex items-center justify-center">
              <HelloGlow className="w-full max-w-2xl rounded-xl">
                <div className="text-center space-y-4 py-24">
                  <h2 className="text-4xl font-bold text-foreground">
                    Premium Animations
                  </h2>
                  <p className="text-muted-foreground">
                    Experience the HelloGlow effect
                  </p>
                </div>
              </HelloGlow>
            </div>

            {/* Feature Cards Grid */}
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 max-w-4xl mx-auto">
              {featureCards.map((card, index) => {
                const Icon = card.icon;
                return (
                  <motion.div
                    key={card.id}
                    initial={{ opacity: 0, y: 20 }}
                    animate={{ opacity: 1, y: 0 }}
                    transition={{ delay: index * 0.1 }}
                  >
                    <Card className="h-full border-border bg-card hover:bg-accent/50 transition-colors cursor-pointer group">
                      <CardHeader className="pb-3">
                        <div className="flex items-center gap-3">
                          <div className="p-2 rounded-lg bg-primary/10 group-hover:bg-primary/20 transition-colors">
                            <Icon className="h-4 w-4 text-primary" />
                          </div>
                          <CardTitle className="text-sm font-medium text-card-foreground">
                            {card.title}
                          </CardTitle>
                        </div>
                      </CardHeader>
                      <CardContent className="pt-0">
                        <p className="text-xs text-muted-foreground leading-relaxed">
                          {card.description}
                        </p>
                      </CardContent>
                    </Card>
                  </motion.div>
                );
              })}
            </div>
            </>
            )}

            {/* Chat Messages - Shown when there are messages */}
            {hasMessages && (
              <div className="space-y-4 p-4 flex-1 overflow-y-auto">
                {messages.map((message) => (
                  <motion.div
                    key={message.id}
                    initial={{ opacity: 0, y: 10 }}
                    animate={{ opacity: 1, y: 0 }}
                    className={cn(
                      "flex w-full gap-3",
                      message.role === "user" ? "justify-end" : "justify-start",
                    )}
                  >
                    {message.role === "user" ? (
                      <>
                        <div className="max-w-[75%] rounded-2xl bg-primary px-4 py-3 text-primary-foreground">
                          <div className="whitespace-pre-wrap wrap-break-word text-sm leading-relaxed">
                            {message.content}
                          </div>
                        </div>
                        <Avatar className="h-8 w-8 shrink-0">
                          <AvatarFallback className="bg-primary text-primary-foreground text-xs font-medium">
                            U
                          </AvatarFallback>
                        </Avatar>
                      </>
                    ) : (
                      <div className="group w-full">
                        <div className="relative w-full bg-transparent">
                          <div className="prose prose-sm prose-slate dark:prose-invert max-w-none select-text leading-relaxed text-foreground">
                            <ReactMarkdown>{message.content}</ReactMarkdown>
                          </div>

                          {message.content && (
                            <div className="mt-3 flex items-center gap-2 opacity-0 transition-opacity group-hover:opacity-100">
                              <Button
                                variant="ghost"
                                size="sm"
                                className="h-7 px-2 gap-1.5 hover:bg-accent"
                                onClick={() =>
                                  navigator.clipboard.writeText(message.content)
                                }
                                title="Copy response"
                              >
                                <Copy className="h-3 w-3" />
                                <span className="text-xs">Copy</span>
                              </Button>
                              <Button
                                variant="ghost"
                                size="sm"
                                className="h-7 px-2 gap-1.5 hover:bg-accent"
                                title="Like response"
                              >
                                <ThumbsUp className="h-3 w-3" />
                                <span className="text-xs">Like</span>
                              </Button>
                              <Button
                                variant="ghost"
                                size="sm"
                                className="h-7 px-2 gap-1.5 hover:bg-accent"
                                title="Dislike response"
                              >
                                <ThumbsDown className="h-3 w-3" />
                                <span className="text-xs">Dislike</span>
                              </Button>
                              <span className="text-xs text-muted-foreground ml-auto">
                                Response time: {(Math.floor(Math.random() * 20 + 5) / 10).toFixed(1)}s
                              </span>
                            </div>
                          )}
                        </div>
                      </div>
                    )}
                  </motion.div>
                ))}
                
                {/* Thinking state - only show when loading and last message is not from assistant or assistant message is empty */}
                {isLoading && (!messages.length || messages[messages.length - 1]?.role !== "assistant" || !messages[messages.length - 1]?.content) && (
                  <motion.div
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    className="flex w-full justify-start"
                  >
                    <span className="text-sm bg-linear-to-r from-muted-foreground via-foreground to-muted-foreground bg-size-[200%_100%] animate-shimmer bg-clip-text text-transparent">
                      Thinking...
                    </span>
                  </motion.div>
                )}
                
                <div ref={messagesEndRef} />
              </div>
            )}
          </div>
        </ScrollArea>
      </motion.div>

      {/* Fixed AI Chat Input - Centered relative to viewport, accounting for sidebar */}
      <div 
        className="fixed bottom-4 z-50 flex justify-center transition-all duration-200"
        style={{
          left: `${sidebarWidth}px`,
          right: 0,
        }}
      >
        <div className="w-full max-w-4xl px-4">
          <AIInputBar 
            messages={messages} 
            onMessagesChange={setMessages}
            isLoading={isLoading}
            onLoadingChange={setIsLoading}
          />
        </div>
      </div>
    </>
  );
}
