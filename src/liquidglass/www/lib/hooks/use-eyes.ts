/* eslint-disable */
// @ts-nocheck
"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import { computeFrame, eyeStateToCSS, type Frame } from "@/liquidglass/www/lib/eyes-engine";
import { DEFAULT_EYE_STATE, type Animation } from "@/liquidglass/www/lib/eyes-types";
import { getAllAnimations, getAnimation, getAnimationsByTrigger, getAnimationByKey } from "@/liquidglass/www/lib/eyes-registry";

export interface UseEyesReturn {
  leftStyle: React.CSSProperties;
  rightStyle: React.CSSProperties;
  gap: number;
  play: (name: string) => void;
  current: string | null;
  isPlaying: boolean;
  animations: Animation[];
  mouseOffset: { x: number; y: number };
  stageProps: {
    onMouseMove: (e: React.MouseEvent) => void;
    onClick: (e: React.MouseEvent) => void;
    onDoubleClick: (e: React.MouseEvent) => void;
    onMouseEnter: () => void;
  };
}

export function useEyes(): UseEyesReturn {
  const [current, setCurrent] = useState<string | null>(null);
  const [frame, setFrame] = useState<Frame>({
    left: DEFAULT_EYE_STATE,
    right: DEFAULT_EYE_STATE,
    gap: 60,
  });
  const [mouseOffset, setMouseOffset] = useState({ x: 0, y: 0 });
  const rafRef = useRef<number>(0);
  const startTimeRef = useRef<number>(0);
  const animRef = useRef<Animation | null>(null);
  const isPlayingRef = useRef(false);

  const play = useCallback((name: string) => {
    const anim = getAnimation(name);
    if (!anim) return;

    if (isPlayingRef.current && animRef.current) {
      const currentPriority = animRef.current.priority ?? 0;
      const newPriority = anim.priority ?? 0;
      if (newPriority < currentPriority) return;
    }

    if (rafRef.current) cancelAnimationFrame(rafRef.current);
    animRef.current = anim;
    isPlayingRef.current = true;
    setCurrent(name);
    startTimeRef.current = performance.now();

    const tick = (now: number) => {
      const elapsed = now - startTimeRef.current;
      if (elapsed >= anim.duration) {
        const finalFrame = computeFrame(anim, anim.duration);
        setFrame(finalFrame);
        isPlayingRef.current = false;
        animRef.current = null;
        setCurrent(null);
        return;
      }
      const f = computeFrame(anim, elapsed);
      setFrame(f);
      rafRef.current = requestAnimationFrame(tick);
    };
    rafRef.current = requestAnimationFrame(tick);
  }, []);

  useEffect(() => {
    const scheduleBlink = () => {
      const delay = 2000 + Math.random() * 4000;
      return setTimeout(() => {
        if (!isPlayingRef.current) {
          play("blink");
        }
        timerId = scheduleBlink();
      }, delay);
    };
    let timerId = scheduleBlink();
    return () => clearTimeout(timerId);
  }, [play]);

  const stageProps = {
    onMouseMove: (e: React.MouseEvent) => {
      const rect = e.currentTarget.getBoundingClientRect();
      const mx = (e.clientX - rect.left) / rect.width - 0.5;
      const my = (e.clientY - rect.top) / rect.height - 0.5;
      setMouseOffset({ x: mx * 18, y: my * 12 });
    },
    onClick: () => {
      const anims = getAnimationsByTrigger("click");
      if (anims[0]) play(anims[0].name);
    },
    onDoubleClick: (e: React.MouseEvent) => {
      e.preventDefault();
      const anims = getAnimationsByTrigger("dblclick");
      if (anims[0]) play(anims[0].name);
    },
    onMouseEnter: () => {
      const anims = getAnimationsByTrigger("mouseenter");
      if (anims[0]) play(anims[0].name);
    },
  };

  const baseLeft = eyeStateToCSS(frame.left);
  const baseRight = eyeStateToCSS(frame.right);

  const applyTracking = (style: React.CSSProperties): React.CSSProperties => {
    if (isPlayingRef.current) return style;
    return {
      ...style,
      transform: `translate(${mouseOffset.x}px, ${mouseOffset.y}px) scaleX(1) scaleY(1) rotate(0deg)`,
    };
  };

  return {
    leftStyle: applyTracking(baseLeft),
    rightStyle: applyTracking(baseRight),
    gap: frame.gap,
    play,
    current,
    isPlaying: isPlayingRef.current,
    animations: getAllAnimations(),
    mouseOffset,
    stageProps,
  };
}
