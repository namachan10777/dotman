import { createCanvas } from "canvas";
import * as fs from "fs";

const width = 3840;
const height = 2160;
const scale = 3.0;

function renderCircle(
  ctx: CanvasRenderingContext2D,
  r1: number,
  r2: number,
  theta_step: number,
  theta_diff: number,
  theta_offset: number,
  color: string
) {
  ctx.strokeStyle = color;
  let theta = 0;
  while (theta + 0.001 < 2 * Math.PI) {
    const x1 = r1 * Math.cos(theta_offset + theta);
    const y1 = r1 * Math.sin(theta_offset + theta);
    const x2 = r2 * Math.cos(theta_offset + theta + theta_step);
    const y2 = r2 * Math.sin(theta_offset + theta + theta_step);
    ctx.beginPath();
    ctx.moveTo(x1 + width / 2, y1 + height / 2);
    ctx.lineTo(x2 + width / 2, y2 + height / 2);
    ctx.stroke();
    theta += theta_diff;
  }
}

function render() {
  const canvas = createCanvas(width, height);
  const ctx = canvas.getContext("2d");
  if (ctx) {
    ctx.fillStyle = "rgb(35, 35, 35)";
    ctx.fillRect(0, 0, width, height);
    renderCircle(
      ctx,
      70 * scale,
      200 * scale,
      (Math.PI / 180) * 40,
      (Math.PI / 180) * 10,
      0,
      "rgb(200, 200, 200, 0.4)"
    );
    renderCircle(
      ctx,
      100 * scale,
      230 * scale,
      (-Math.PI / 180) * 40,
      (Math.PI / 180) * 10,
      0.5,
      "rgb(200, 200, 200, 0.4)"
    );
    renderCircle(
      ctx,
      110 * scale,
      250 * scale,
      (-Math.PI / 180) * 20,
      (Math.PI / 180) * 10,
      0.9,
      "rgb(200, 200, 200, 0.4)"
    );
  }
  return canvas.toBuffer();
}

const buffer = render();
fs.writeFileSync("wallpaper.png", buffer);
