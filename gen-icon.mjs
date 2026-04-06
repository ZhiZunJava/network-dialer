import { writeFileSync } from "fs";
import { deflateSync } from "zlib";

const SIZE = 1024;
const RADIUS = 200; // corner radius
const PADDING = 40; // padding from edge for the rounded rect

// Colors
const GRAD_TOP = [37, 99, 235];    // #2563eb
const GRAD_BOTTOM = [22, 119, 255]; // #1677ff
const SHADOW_COLOR = [15, 60, 150]; // darker blue for shadow
const WHITE = [255, 255, 255];

function lerp(a, b, t) {
  return a + (b - a) * t;
}

function lerpColor(c1, c2, t) {
  return [
    Math.round(lerp(c1[0], c2[0], t)),
    Math.round(lerp(c1[1], c2[1], t)),
    Math.round(lerp(c1[2], c2[2], t)),
  ];
}

function clamp(v, min, max) {
  return Math.max(min, Math.min(max, v));
}

// Signed distance from point to rounded rectangle
// rect centered at (cx,cy) with half-sizes (hw,hh) and corner radius r
function sdRoundedRect(px, py, cx, cy, hw, hh, r) {
  const dx = Math.abs(px - cx) - hw + r;
  const dy = Math.abs(py - cy) - hh + r;
  const outsideDist = Math.sqrt(Math.max(dx, 0) ** 2 + Math.max(dy, 0) ** 2) - r;
  const insideDist = Math.min(Math.max(dx, dy), 0) - r;
  // negative = inside, 0 = on edge, positive = outside
  return outsideDist > 0 ? outsideDist : insideDist;
}

// Check if a point is inside an arc ring segment
// Arc centered at (cx, cy), between innerR and outerR,
// spanning angles from startAngle to endAngle (radians, measured from top, going CW)
function inArc(px, py, cx, cy, innerR, outerR, halfAngle) {
  const dx = px - cx;
  const dy = py - cy;
  const dist = Math.sqrt(dx * dx + dy * dy);
  if (dist < innerR || dist > outerR) return -1;
  // Angle from top (negative Y axis)
  const angle = Math.atan2(dx, -dy); // 0 = up, positive = clockwise
  if (Math.abs(angle) > halfAngle) return -1;
  // Return distance to nearest edge for antialiasing
  const distToInner = dist - innerR;
  const distToOuter = outerR - dist;
  const angleEdgeDist = (halfAngle - Math.abs(angle)) * dist;
  return Math.min(distToInner, distToOuter, angleEdgeDist);
}

function createPNG(width, height) {
  const signature = Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]);

  function crc32(buf) {
    let c = 0xffffffff;
    const table = new Int32Array(256);
    for (let n = 0; n < 256; n++) {
      let cr = n;
      for (let k = 0; k < 8; k++) {
        cr = cr & 1 ? 0xedb88320 ^ (cr >>> 1) : cr >>> 1;
      }
      table[n] = cr;
    }
    for (let i = 0; i < buf.length; i++) {
      c = table[(c ^ buf[i]) & 0xff] ^ (c >>> 8);
    }
    return (c ^ 0xffffffff) >>> 0;
  }

  function chunk(type, data) {
    const len = Buffer.alloc(4);
    len.writeUInt32BE(data.length);
    const typeData = Buffer.concat([Buffer.from(type), data]);
    const crc = Buffer.alloc(4);
    crc.writeUInt32BE(crc32(typeData));
    return Buffer.concat([len, typeData, crc]);
  }

  // IHDR - RGBA (color type 6)
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(width, 0);
  ihdr.writeUInt32BE(height, 4);
  ihdr[8] = 8;  // bit depth
  ihdr[9] = 6;  // color type RGBA
  ihdr[10] = 0; // compression
  ihdr[11] = 0; // filter
  ihdr[12] = 0; // interlace

  // Pixel data - RGBA (4 bytes per pixel)
  const rawData = Buffer.alloc(height * (1 + width * 4));

  // Rounded rect dimensions
  const rectLeft = PADDING;
  const rectTop = PADDING;
  const rectRight = width - PADDING;
  const rectBottom = height - PADDING;
  const rectCX = width / 2;
  const rectCY = height / 2;
  const rectHW = (rectRight - rectLeft) / 2;
  const rectHH = (rectBottom - rectTop) / 2;

  // WiFi symbol parameters
  const wifiCX = width / 2;
  const wifiCY = height / 2 + 60; // slightly below center (dot position)
  const dotRadius = 48;

  // Arc parameters
  const arcThickness = 62;
  const arcGap = 32;
  const halfAngle = Math.PI / 3.3; // ~55 degrees each side

  // Three arcs
  const arcs = [
    { inner: dotRadius + arcGap, outer: dotRadius + arcGap + arcThickness },
    { inner: dotRadius + arcGap + arcThickness + arcGap, outer: dotRadius + arcGap + arcThickness + arcGap + arcThickness },
    { inner: dotRadius + arcGap + (arcThickness + arcGap) * 2, outer: dotRadius + arcGap + (arcThickness + arcGap) * 2 + arcThickness },
  ];

  // Shadow offset
  const shadowOffsetY = 12;
  const shadowBlur = 30;

  for (let y = 0; y < height; y++) {
    const offset = y * (1 + width * 4);
    rawData[offset] = 0; // filter none

    for (let x = 0; x < width; x++) {
      const px = offset + 1 + x * 4;

      // Start with fully transparent
      let r = 0, g = 0, b = 0, a = 0;

      // --- Shadow layer ---
      const shadowDist = sdRoundedRect(x, y - shadowOffsetY, rectCX, rectCY, rectHW, rectHH, RADIUS);
      if (shadowDist < shadowBlur) {
        const shadowAlpha = clamp(1 - shadowDist / shadowBlur, 0, 1);
        const sa = Math.round(shadowAlpha * 0.35 * 255);
        r = SHADOW_COLOR[0];
        g = SHADOW_COLOR[1];
        b = SHADOW_COLOR[2];
        a = sa;
      }

      // --- Rounded rectangle background ---
      const dist = sdRoundedRect(x, y, rectCX, rectCY, rectHW, rectHH, RADIUS);
      if (dist < 1.0) {
        // Antialiased edge
        const edgeAlpha = clamp(1 - dist, 0, 1);

        // Gradient from top to bottom
        const t = clamp((y - rectTop) / (rectBottom - rectTop), 0, 1);
        const bgColor = lerpColor(GRAD_TOP, GRAD_BOTTOM, t);

        // Subtle inner glow at top for depth
        const topGlow = clamp(1 - (y - rectTop) / (rectHH * 0.6), 0, 1);
        const glowStrength = topGlow * 0.15;
        const finalR = Math.round(bgColor[0] + (255 - bgColor[0]) * glowStrength);
        const finalG = Math.round(bgColor[1] + (255 - bgColor[1]) * glowStrength);
        const finalB = Math.round(bgColor[2] + (255 - bgColor[2]) * glowStrength);

        const bgAlpha = Math.round(edgeAlpha * 255);

        // Composite over shadow
        const srcA = bgAlpha / 255;
        const dstA = a / 255;
        const outA = srcA + dstA * (1 - srcA);
        if (outA > 0) {
          r = Math.round((finalR * srcA + r * dstA * (1 - srcA)) / outA);
          g = Math.round((finalG * srcA + g * dstA * (1 - srcA)) / outA);
          b = Math.round((finalB * srcA + b * dstA * (1 - srcA)) / outA);
          a = Math.round(outA * 255);
        }
      }

      // --- White WiFi symbol (only draw if inside the rounded rect) ---
      if (dist < 0) {
        let wifiAlpha = 0;

        // Dot at bottom center
        const dotDx = x - wifiCX;
        const dotDy = y - wifiCY;
        const dotDist = Math.sqrt(dotDx * dotDx + dotDy * dotDy);
        if (dotDist < dotRadius + 1.5) {
          wifiAlpha = clamp(dotRadius + 1.5 - dotDist, 0, 1);
        }

        // Arcs
        for (const arc of arcs) {
          const d = inArc(x, y, wifiCX, wifiCY, arc.inner, arc.outer, halfAngle);
          if (d >= 0) {
            const arcAlpha = clamp(d + 1, 0, 1); // AA at edges
            wifiAlpha = Math.max(wifiAlpha, Math.min(arcAlpha, 1));
          }
        }

        // Composite white WiFi over background
        if (wifiAlpha > 0) {
          const wa = wifiAlpha;
          r = Math.round(WHITE[0] * wa + r * (1 - wa));
          g = Math.round(WHITE[1] * wa + g * (1 - wa));
          b = Math.round(WHITE[2] * wa + b * (1 - wa));
          // Alpha stays the same (background is opaque here)
        }
      }

      rawData[px] = clamp(r, 0, 255);
      rawData[px + 1] = clamp(g, 0, 255);
      rawData[px + 2] = clamp(b, 0, 255);
      rawData[px + 3] = clamp(a, 0, 255);
    }
  }

  const compressed = deflateSync(rawData, { level: 9 });

  return Buffer.concat([
    signature,
    chunk("IHDR", ihdr),
    chunk("IDAT", compressed),
    chunk("IEND", Buffer.alloc(0)),
  ]);
}

const png = createPNG(SIZE, SIZE);
writeFileSync("app-icon.png", png);
console.log("Icon generated: app-icon.png (" + png.length + " bytes)");
