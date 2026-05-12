"use client";

import { motion } from "motion/react";
import { useEffect, useRef, useState } from "react";

interface GlassCrackProps {
  show: boolean;
  targetX: number;
  targetY: number;
}

interface Point {
  x: number;
  y: number;
}

interface CrackLine {
  p1: Point;
  p2: Point;
  desc: any;
  level: number;
}

const RAD = Math.PI / 180;

function findPointOnCircle(c: Point, r: number, a: number): Point {
  return {
    x: c.x + r * Math.cos(a * RAD) - r * Math.sin(a * RAD),
    y: c.y + r * Math.sin(a * RAD) + r * Math.cos(a * RAD),
  };
}

function describeLinePath(p1: Point, p2: Point, cv: number) {
  const o: any = {};
  cv = 5 * cv;
  
  o.dx = p2.x - p1.x;
  o.dy = p2.y - p1.y;
  o.dl = Math.sqrt(o.dx * o.dx + o.dy * o.dy);
  
  o.sx = o.dx / o.dl;
  o.sy = o.dy / o.dl;
  o.tx = o.dy / o.dl;
  o.ty = -o.dx / o.dl;
  
  o.mpp = Math.random() * 0.5 + 0.3;
  o.mpl1 = o.dl * o.mpp;
  o.mpl2 = o.dl - o.mpl1;
  
  const ll = Math.log(o.dl * Math.E);
  o.cma = Math.random() * ll * cv - (ll * cv) / 2;
  o.cpt = {
    x: p1.x + o.sx * o.mpl1 + o.tx * o.cma,
    y: p1.y + o.sy * o.mpl1 + o.ty * o.cma,
  };
  
  o.bbx1 = Math.min(p1.x, p2.x, o.cpt.x);
  o.bby1 = Math.min(p1.y, p2.y, o.cpt.y);
  o.bbx2 = Math.max(p1.x, p2.x, o.cpt.x);
  o.bby2 = Math.max(p1.y, p2.y, o.cpt.y);
  o.bbwidth = o.bbx2 - o.bbx1;
  o.bbheight = o.bby2 - o.bby1;
  
  return o;
}

function findCrackPaths(center: Point, width: number, height: number): CrackLine[] {
  const main: any[][] = [[]];
  const lines: CrackLine[] = [];
  let level = 1;
  let maxl = 0;
  let r = 15;
  const num = 8; // Reduced from 12 to lower intensity further
  const ang = 360 / (num + 1);
  
  while (main[0].length < num) {
    const num2 = ang * main[0].length + 10;
    const pt2 = findPointOnCircle(center, 5, num2);
    main[0].push({ angle: num2, point: pt2 });
  }
  
  while (r < 500) {
    main[level] = [];
    for (let num2 = 0; num2 < num; num2++) {
      const pt1 = main[level - 1][num2];
      main[level][num2] = null;
      
      if (pt1) {
        if (pt1.point.x > 0 && pt1.point.x < width && pt1.point.y > 0 && pt1.point.y < height) {
          let angle = pt1.angle + (Math.random() * 10) / num - 10 / 2 / num;
          if (angle > 350) angle = 350;
          
          const pt2 = findPointOnCircle(
            center,
            r + Math.random() * r / level - r / (level * 2),
            angle
          );
          
          main[level][num2] = {
            angle: angle,
            point: { x: pt2.x, y: pt2.y },
          };
        } else if (maxl === 0) {
          maxl = level;
        }
      }
    }
    level++;
    r *= Math.random() * 1.5 + 1;
  }
  
  if (maxl === 0) maxl = level;
  
  for (let l = 1; l < level; l++) {
    for (let g = 0; g < num; g++) {
      const pt1 = main[l - 1][g];
      const pt2 = main[l][g];
      
      if (pt1 && pt2) {
        lines.push({
          p1: { x: pt1.point.x, y: pt1.point.y },
          p2: { x: pt2.point.x, y: pt2.point.y },
          desc: describeLinePath(pt1.point, pt2.point, 0.3),
          level: l,
        });
        
        if (Math.random() < 0.2) { // Reduced from 0.3
          const pt3 = main[l][(g + 1) % num];
          if (pt3) {
            lines.push({
              p1: { x: pt2.point.x, y: pt2.point.y },
              p2: { x: pt3.point.x, y: pt3.point.y },
              desc: describeLinePath(pt2.point, pt3.point, 0.3),
              level: l,
            });
          }
        }
        
        if (l < level - 1 && Math.random() < 0.1) { // Reduced from 0.15
          const pt3 = main[l + 1][(g + 1) % num];
          if (pt3) {
            lines.push({
              p1: { x: pt2.point.x, y: pt2.point.y },
              p2: { x: pt3.point.x, y: pt3.point.y },
              desc: describeLinePath(pt2.point, pt3.point, 0.3),
              level: l,
            });
          }
        }
      }
    }
  }
  
  return lines;
}

export function GlassCrack({ show, targetX, targetY }: GlassCrackProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [hasRendered, setHasRendered] = useState(false);

  useEffect(() => {
    if (show && canvasRef.current && !hasRendered) {
      setHasRendered(true);
      const canvas = canvasRef.current;
      const ctx = canvas.getContext("2d");
      if (!ctx) return;

      canvas.width = window.innerWidth;
      canvas.height = window.innerHeight;

      const center = { x: targetX, y: targetY };
      const lines = findCrackPaths(center, canvas.width, canvas.height);

      // Render main crack lines
      lines.forEach((line) => {
        const delay = line.level * 15;
        setTimeout(() => {
          const { p1, p2, desc } = line;
          const { tx, ty, dl, cpt } = desc;

          // Main crack line - clean glass crack appearance
          ctx.globalAlpha = 0.5;
          ctx.strokeStyle = "rgba(200, 200, 200, 0.8)";
          ctx.lineWidth = 1;
          ctx.beginPath();
          ctx.moveTo(p1.x, p1.y);
          ctx.quadraticCurveTo(cpt.x, cpt.y, p2.x, p2.y);
          ctx.stroke();

          // Glass reflection effect along crack
          ctx.globalAlpha = 0.25;
          const dd = dl / 4;
          const grd = ctx.createLinearGradient(
            p1.x + dd * tx,
            p1.y + dd * ty,
            p1.x - dd * tx,
            p1.y - dd * ty
          );
          grd.addColorStop(0, "rgba(255,255,255,0)");
          grd.addColorStop(0.5, "rgba(255,255,255,0.4)");
          grd.addColorStop(1, "rgba(255,255,255,0)");
          ctx.fillStyle = grd;
          ctx.beginPath();
          ctx.moveTo(p1.x + dd * tx, p1.y + dd * ty);
          ctx.lineTo(p2.x + dd * tx, p2.y + dd * ty);
          ctx.lineTo(p2.x - dd * tx, p2.y - dd * ty);
          ctx.lineTo(p1.x - dd * tx, p1.y - dd * ty);
          ctx.closePath();
          ctx.fill();
        }, delay);
      });
    } else if (!show && canvasRef.current && hasRendered) {
      // Clear canvas when animation ends
      const canvas = canvasRef.current;
      const ctx = canvas.getContext("2d");
      if (ctx) {
        ctx.clearRect(0, 0, canvas.width, canvas.height);
      }
      setHasRendered(false);
    }
  }, [show, targetX, targetY, hasRendered]);

  if (!show) return null;

  return (
    <div className="pointer-events-none fixed inset-0 z-50">
      {/* Sniper red dot */}
      <motion.div
        initial={{ opacity: 0, scale: 0 }}
        animate={{ opacity: [0, 1, 1, 0], scale: [0, 1, 1, 0] }}
        transition={{ duration: 0.6, times: [0, 0.05, 0.95, 1] }}
        className="absolute"
        style={{
          left: targetX,
          top: targetY,
          transform: "translate(-50%, -50%)",
        }}
      >
        <div className="absolute w-3 h-3 rounded-full bg-red-600 shadow-[0_0_20px_rgba(220,38,38,1)] left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2">
          <div className="absolute inset-0 rounded-full bg-red-600 animate-ping opacity-75" />
        </div>
        <div className="absolute w-16 h-px bg-red-500/70 left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2" />
        <div className="absolute h-16 w-px bg-red-500/70 left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2" />
        <div className="absolute w-20 h-20 border border-red-500/50 rounded-full left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2" />
      </motion.div>

      {/* Canvas for crack rendering */}
      <canvas
        ref={canvasRef}
        className="absolute inset-0 w-full h-full"
        style={{ mixBlendMode: "multiply" }}
      />

      {/* Impact flash */}
      <motion.div
        initial={{ opacity: 0, scale: 0 }}
        animate={{ opacity: [0, 0.8, 0], scale: [0, 4, 6] }}
        transition={{ duration: 0.4 }}
        className="absolute w-32 h-32 rounded-full bg-white/40 blur-3xl"
        style={{
          left: targetX,
          top: targetY,
          transform: "translate(-50%, -50%)",
        }}
      />

      {/* Screen darkening - stays visible longer */}
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: [0, 0.15, 0.12, 0.12, 0] }}
        transition={{ duration: 3.5, times: [0, 0.05, 0.2, 0.85, 1] }}
        className="absolute inset-0"
        style={{
          background: `radial-gradient(circle at ${targetX}px ${targetY}px, transparent 0%, rgba(0,0,0,0.15) 50%, rgba(0,0,0,0.3) 100%)`,
        }}
      />
    </div>
  );
}
