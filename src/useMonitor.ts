import { useEffect, useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { type Bootstrap, type Snapshot, type Settings, emptySnapshot, defaultSettings } from './model';

export function useMonitor() {
  const [snapshot, setSnapshot] = useState(emptySnapshot);
  const [settings, setSettings] = useState(defaultSettings);
  const [error, setError] = useState<string | null>(null);
  const [ready, setReady] = useState(false);
  useEffect(() => {
    let alive = true; const unlisten: UnlistenFn[] = [];
    async function start() {
      try {
        for (const [event, handler] of [
          ['snapshot', (p: unknown) => setSnapshot(p as Snapshot)],
          ['settings', (p: unknown) => setSettings(p as Settings)],
          ['runtime-error', (p: unknown) => setError(String(p))],
        ] as const) {
          const stop = await listen(event, e => { if (alive) handler(e.payload); });
          if (!alive) {stop(); return;} unlisten.push(stop);
        }
        const data = await invoke<Bootstrap>('bootstrap');
        if (alive) {setSnapshot(data.snapshot); setSettings(data.settings); setError(data.error); setReady(true);}
      } catch { if (alive) setError('Native connection unavailable. Launch the desktop application.'); }
    }
    void start(); return () => {alive = false; unlisten.forEach(stop => stop());};
  }, []);
  const save = useCallback(async (next: Settings) => {
    try {const saved = await invoke<Settings>('save_settings', {settings: next});setSettings(saved);setError(null);return true;}
    catch (e) {setError(String(e));return false;}
  }, []);
  return {snapshot, settings, error, ready, save};
}
