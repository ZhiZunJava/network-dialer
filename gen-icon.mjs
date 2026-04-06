import { writeFileSync } from "fs";
import { deflateSync } from "zlib";

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

  // IHDR
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(width, 0);
  ihdr.writeUInt32BE(height, 4);
  ihdr[8] = 8; // bit depth
  ihdr[9] = 2; // color type RGB
  ihdr[10] = 0;
  ihdr[11] = 0;
  ihdr[12] = 0;

  // Pixel data
  const rawData = Buffer.alloc(height * (1 + width * 3));
  for (let y = 0; y < height; y++) {
    const offset = y * (1 + width * 3);
    rawData[offset] = 0; // filter none
    for (let x = 0; x < width; x++) {
      const px = offset + 1 + x * 3;
      const cx = width / 2,
        cy = height / 2;
      const dx = x - cx,
        dy = y - cy;
      const dist = Math.sqrt(dx * dx + dy * dy);
      const maxDist = width / 2;

      if (dist < maxDist * 0.85) {
        const t = dist / (maxDist * 0.85);
        rawData[px] = Math.floor(22 + t * 20);
        rawData[px + 1] = Math.floor(119 + t * 30);
        rawData[px + 2] = 255;
      } else if (dist < maxDist * 0.9) {
        rawData[px] = 255;
        rawData[px + 1] = 255;
        rawData[px + 2] = 255;
      } else {
        rawData[px] = 240;
        rawData[px + 1] = 240;
        rawData[px + 2] = 240;
      }

      // WiFi arcs
      const angle = Math.atan2(dy, dx);
      const angleFromTop = Math.abs(angle + Math.PI / 2);
      if (angleFromTop < 0.6) {
        if (
          (dist > maxDist * 0.55 && dist < maxDist * 0.65) ||
          (dist > maxDist * 0.35 && dist < maxDist * 0.45) ||
          (dist > maxDist * 0.15 && dist < maxDist * 0.25)
        ) {
          rawData[px] = 255;
          rawData[px + 1] = 255;
          rawData[px + 2] = 255;
        }
      }
      if (dist < maxDist * 0.08) {
        rawData[px] = 255;
        rawData[px + 1] = 255;
        rawData[px + 2] = 255;
      }
    }
  }

  const compressed = deflateSync(rawData);

  return Buffer.concat([
    signature,
    chunk("IHDR", ihdr),
    chunk("IDAT", compressed),
    chunk("IEND", Buffer.alloc(0)),
  ]);
}

const png = createPNG(1024, 1024);
writeFileSync("app-icon.png", png);
console.log("Icon generated: app-icon.png (" + png.length + " bytes)");
