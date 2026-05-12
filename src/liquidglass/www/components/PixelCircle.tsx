import React, { useRef, useEffect } from 'react';

interface PixelCircleProps {
  palette: string[];
  speed: number;
  resolution: number;
  isPlaying: boolean;
  timeOffset: number;
  size: number;
  noiseAmount: number;
}

export const PixelCircle: React.FC<PixelCircleProps> = ({ 
  palette, 
  speed, 
  resolution, 
  isPlaying, 
  timeOffset, 
  size,
  noiseAmount
}) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const requestRef = useRef<number | undefined>(undefined);
  const timeRef = useRef<number>(timeOffset);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const offscreen = document.createElement('canvas');
    const octx = offscreen.getContext('2d');
    if (!octx) return;

    const render = () => {
      if (isPlaying) {
        timeRef.current += speed * 0.01;
      }

      offscreen.width = resolution;
      offscreen.height = resolution;

      const scale = 6 / resolution;

      for (let y = 0; y < resolution; y++) {
        for (let x = 0; x < resolution; x++) {
          const t = timeRef.current;
          
          // Organic noise using combined sine waves
          const noise = (
            Math.sin(x * scale + t) +
            Math.sin(y * scale + t * 0.8) +
            Math.sin((x + y) * scale * 0.7 - t * 0.6) +
            Math.sin((x - y) * scale * 0.7 + t * 0.4)
          ) / 4;
          
          let normalized = (noise + 1) / 2;
          
          // Add static noise for texture
          if (noiseAmount > 0) {
            normalized += (Math.random() - 0.5) * noiseAmount;
          }
          
          const index = Math.floor(normalized * palette.length);
          const clampedIndex = Math.max(0, Math.min(palette.length - 1, index));
          
          octx.fillStyle = palette[clampedIndex];
          octx.fillRect(x, y, 1, 1);
        }
      }

      // Disable smoothing for pixelated effect
      ctx.imageSmoothingEnabled = false;
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      ctx.drawImage(offscreen, 0, 0, canvas.width, canvas.height);

      if (isPlaying) {
        requestRef.current = requestAnimationFrame(render);
      }
    };

    if (!isPlaying) {
      render();
    } else {
      requestRef.current = requestAnimationFrame(render);
    }

    return () => {
      if (requestRef.current) cancelAnimationFrame(requestRef.current);
    };
  }, [palette, speed, resolution, isPlaying, size, noiseAmount]);

  return (
    <canvas 
      ref={canvasRef} 
      width={size} 
      height={size} 
      style={{ width: size, height: size }}
      className="block bg-black" 
    />
  );
};
