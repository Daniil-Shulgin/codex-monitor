import type { Task } from './model';
const pad = (n: number) => String(n).padStart(2, '0');
export function resetTime(at: number | null, now: number): string {
  if (at === null) return '—';
  const seconds = Math.max(0, Math.ceil(at - now / 1000));
  if (seconds === 0) return '0m';
  const minutes = Math.ceil(seconds / 60);
  if (minutes < 60) return `${minutes}m`;
  const hours = Math.floor(minutes / 60);
  if (hours >= 24) return `${Math.floor(hours / 24)}d${hours % 24}h`;
  return `${pad(hours)}:${pad(minutes % 60)}`;
}
export function elapsed(task: Task, now: number): string {
  const milliseconds = task.status === 'done' ? task.durationMs ?? (task.completedAt ?? now) - task.startedAt : now - task.startedAt;
  const seconds = Math.max(0, Math.floor(milliseconds / 1000));
  const hours = Math.floor(seconds / 3600);
  return hours ? `${hours}:${pad(Math.floor(seconds / 60) % 60)}:${pad(seconds % 60)}` : `${pad(Math.floor(seconds / 60))}:${pad(seconds % 60)}`;
}
