/* eslint-disable */
// @ts-nocheck
interface Point {
  x: number;
  y: number;
}

/**
 * Generates an SVG path string for a glass crack pattern
 * that can be used as a clip-path on elements
 */
export function generateCrackClipPath(
  centerX: number,
  centerY: number,
  width: number,
  height: number,
  branches = 8
): string {
  const paths: string[] = [];
  const angleStep = 360 / branches;
  
  // Create radial cracks from center
  for (let i = 0; i < branches; i++) {
    const angle = (angleStep * i + Math.random() * 20 - 10) * (Math.PI / 180);
    const length = Math.random() * 40 + 30;
    
    // Main crack line
    const endX = centerX + Math.cos(angle) * length;
    const endY = centerY + Math.sin(angle) * length;
    
    // Add some curve variation
    const midX = centerX + Math.cos(angle) * (length / 2) + (Math.random() - 0.5) * 10;
    const midY = centerY + Math.sin(angle) * (length / 2) + (Math.random() - 0.5) * 10;
    
    paths.push(`M ${centerX} ${centerY} Q ${midX} ${midY} ${endX} ${endY}`);
    
    // Add branch cracks
    if (Math.random() > 0.4) {
      const branchAngle = angle + (Math.random() > 0.5 ? 0.5 : -0.5);
      const branchLength = length * 0.6;
      const branchEndX = midX + Math.cos(branchAngle) * branchLength;
      const branchEndY = midY + Math.sin(branchAngle) * branchLength;
      paths.push(`M ${midX} ${midY} L ${branchEndX} ${branchEndY}`);
    }
  }
  
  // Create the full shape with cracks cut out
  const fullRect = `M 0 0 L ${width} 0 L ${width} ${height} L 0 ${height} Z`;
  
  return fullRect + " " + paths.join(" ");
}

/**
 * Generates multiple crack patterns for animation frames
 */
export function generateCrackSequence(
  centerX: number,
  centerY: number,
  width: number,
  height: number,
  frames = 5
): string[] {
  const sequence: string[] = [];
  
  for (let i = 0; i < frames; i++) {
    const branches = 6 + i * 2;
    sequence.push(generateCrackClipPath(centerX, centerY, width, height, branches));
  }
  
  return sequence;
}
