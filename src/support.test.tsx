import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import jsQR from 'jsqr';
import { afterEach, expect, it, vi } from 'vitest';
import { Support, SUPPORT_ADDRESS } from './components/Support';

afterEach(cleanup);

it('opens from Settings, copies the address, and restores focus on close', async () => {
  const writeText = vi.fn(async () => {});
  Object.defineProperty(navigator, 'clipboard', { configurable: true, value: { writeText } });
  render(<Support/>);
  const trigger = screen.getByRole('button', { name: 'Support Codex Monitor' });
  expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
  fireEvent.click(trigger);
  const dialog = screen.getByRole('dialog', { name: 'Support Codex Monitor' });
  expect(screen.getByRole('button', { name: 'Close support' })).toHaveFocus();
  fireEvent.click(screen.getByRole('button', { name: 'Copy address' }));
  await waitFor(() => expect(writeText).toHaveBeenCalledWith(SUPPORT_ADDRESS));
  expect(screen.getByRole('status')).toHaveTextContent('Copied');
  fireEvent.keyDown(dialog, { key: 'Escape' });
  expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
  expect(trigger).toHaveFocus();
  fireEvent.click(trigger);
  fireEvent.click(screen.getByRole('button', { name: 'Close support' }));
  expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
});

it('independently decodes the rendered QR to the exact receiving address', () => {
  render(<Support/>);
  fireEvent.click(screen.getByRole('button', { name: 'Support Codex Monitor' }));
  const svg = screen.getByRole('img', { name: 'USDT TRC20 receiving address' });
  const modules = Number(svg.getAttribute('viewBox')?.split(' ')[2]);
  const scale = 5;
  const width = modules * scale;
  const pixels = new Uint8ClampedArray(width * width * 4).fill(255);
  const path = svg.querySelector('path[fill="#000000"]')?.getAttribute('d') ?? '';
  for (const match of path.matchAll(/M(\d+)[ ,](\d+)\s*h(\d+)v1H\d+z/g)) {
    const [, left, top, run] = match.map(Number);
    for (let y = top * scale; y < (top + 1) * scale; y++) {
      for (let x = left * scale; x < (left + run) * scale; x++) {
        const pixel = (y * width + x) * 4;
        pixels[pixel] = pixels[pixel + 1] = pixels[pixel + 2] = 0;
      }
    }
  }
  expect(jsQR(pixels, width, width)?.data).toBe(SUPPORT_ADDRESS);
});
