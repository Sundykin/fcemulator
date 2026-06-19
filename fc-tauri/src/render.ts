// PixiJS v8 renderer — pixel-perfect at any devicePixelRatio.
//
// The trap: integer scaling in *logical* px still lands on fractional *device*
// px when dpr is non-integer (Mac "scaled" display modes), so nearest sampling
// drops/duplicates columns and thin glyph strokes garble (8 → R, 1-1 → _1_1).
// Fix: scale by an integer number of DEVICE pixels — the backing store is
// 256·s × 240·s device px, its CSS size maps that 1:1 (or stretched for a
// chosen display aspect ratio).
import { Application, Texture, Sprite, BufferImageSource } from "pixi.js";

const W = 256;
const H = 240;

export interface RenderSettings {
  filter: "pixel" | "smooth";
  aspect: "orig" | "square" | "stretch"; // 4:3 / 1:1 / fill
  zoom: "auto" | "2x" | "3x";
}

export interface FcRenderer {
  update(buf: ArrayBuffer): void;
  resize(): void;
  setSettings(s: RenderSettings): void;
  destroy(): void;
}

export async function createRenderer(container: HTMLElement): Promise<FcRenderer> {
  const app = new Application();
  await app.init({ width: W, height: H, background: 0x000000, antialias: false, resolution: 1, autoDensity: false });
  container.appendChild(app.canvas);

  const pixels = new Uint8Array(W * H * 4);
  const source = new BufferImageSource({ resource: pixels, width: W, height: H, scaleMode: "nearest", format: "rgba8unorm" });
  const texture = new Texture({ source });
  const sprite = new Sprite(texture);
  app.stage.addChild(sprite);

  let settings: RenderSettings = { filter: "pixel", aspect: "orig", zoom: "auto" };

  function fit() {
    const dpr = window.devicePixelRatio || 1;
    const cw = container.clientWidth || W;
    const ch = container.clientHeight || H;
    // Largest integer device-pixel scale that fits (square pixels backing store).
    const maxS = Math.max(1, Math.floor(Math.min((cw * dpr) / W, (ch * dpr) / H)));
    let s = maxS;
    if (settings.zoom === "2x") s = Math.min(2, maxS) || 2;
    else if (settings.zoom === "3x") s = Math.min(3, maxS) || 3;
    const bw = W * s;
    const bh = H * s;
    app.renderer.resize(bw, bh);
    sprite.scale.set(s);
    sprite.position.set(0, 0);

    // CSS display size by aspect (backing store stays crisp/square).
    let dw = bw / dpr;
    let dh = bh / dpr;
    if (settings.aspect === "orig") {
      // 4:3 display: stretch width from square, then fit inside the container.
      dw = dh * (4 / 3);
      if (dw > cw) {
        const k = cw / dw;
        dw = cw;
        dh *= k;
      }
    } else if (settings.aspect === "stretch") {
      dw = cw;
      dh = ch;
    }
    app.canvas.style.width = dw + "px";
    app.canvas.style.height = dh + "px";
  }
  fit();

  return {
    update(buf: ArrayBuffer) {
      const u8 = new Uint8Array(buf);
      if (u8.length === pixels.length) {
        pixels.set(u8);
        source.update();
      }
    },
    resize: fit,
    setSettings(s: RenderSettings) {
      settings = s;
      try {
        source.scaleMode = s.filter === "smooth" ? "linear" : "nearest";
        source.update();
      } catch {
        /* scaleMode not settable on this Pixi build */
      }
      fit();
    },
    destroy() {
      app.destroy(true, { children: true, texture: true });
    },
  };
}
