export type TaskStatus = 'running' | 'needsAction' | 'done';
export interface Quota { remainingPercent: number | null; resetsAt: number | null }
export interface Task { threadId: string; turnId: string; displayName: string; status: TaskStatus; startedAt: number; completedAt: number | null; durationMs: number | null; updatedAt: number }
export interface Snapshot { fiveHour: Quota; weekly: Quota; tasks: Task[]; quotaConnected: boolean; ideConnected: boolean; diagnostic: string; quotaUpdatedAt: number | null }
export interface Settings { theme: 'system' | 'dark' | 'light'; opacity: number; pinned: boolean; expanded: boolean; clickThrough: boolean; shortcut: string; launchAtLogin: boolean; notifyAction: boolean; notifyDone: boolean; sound: boolean; volume: number; refreshSeconds: number; position: [number, number] | null }
export interface Bootstrap { snapshot: Snapshot; settings: Settings; error: string | null }
export const emptySnapshot: Snapshot = { fiveHour: {remainingPercent:null,resetsAt:null}, weekly:{remainingPercent:null,resetsAt:null}, tasks:[], quotaConnected:false, ideConnected:false, diagnostic:'Connecting to local Codex…', quotaUpdatedAt:null };
export const defaultSettings: Settings = {theme:'system',opacity:.88,pinned:true,expanded:false,clickThrough:false,shortcut:'Ctrl+Shift+C',launchAtLogin:false,notifyAction:true,notifyDone:true,sound:true,volume:35,refreshSeconds:30,position:null};
