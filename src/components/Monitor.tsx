import type { Settings, Snapshot, Task } from '../model';
import { QuotaIndicator } from './QuotaIndicator';
import { StatusCounters, StatusIcon, statusLabels } from './Status';
import { WindowControls } from './WindowControls';
import { elapsed } from '../format';
import { invoke } from '@tauri-apps/api/core';
import { emit } from '@tauri-apps/api/event';
import type { MouseEvent } from 'react';
import { Settings as SettingsIcon, X } from 'lucide-react';
export interface MonitorProps { snapshot:Snapshot;settings:Settings;now:number;save:(s:Settings)=>Promise<boolean>;error:string|null }
function drag(e: MouseEvent) {if (e.button === 0 && !(e.target as HTMLElement).closest('button')) {e.preventDefault();void invoke('drag_widget').catch(error=>emit('runtime-error',String(error)));}}
function Quotas({snapshot,now}: Pick<MonitorProps,'snapshot'|'now'>) {return <><QuotaIndicator label="5H" quota={snapshot.fiveHour} now={now} stale={!snapshot.quotaConnected}/><QuotaIndicator label="W" quota={snapshot.weekly} now={now} stale={!snapshot.quotaConnected}/></>;}
export function CollapsedMonitor(props: MonitorProps) {
  return <main className="monitor collapsed" onMouseDown={drag} aria-label="Codex Monitor">
    <span className="brand" title={props.error ?? 'Codex Monitor · Drag to move'}>CODEX</span>
    <div className="quotas"><Quotas {...props}/></div>
    {props.snapshot.tasks.length > 0 && <div className="counters"><StatusCounters tasks={props.snapshot.tasks}/></div>}
    <WindowControls {...props}/>
  </main>;
}
export function TaskRow({task,now}: {task:Task;now:number}) {
  return <div className="task-row" role="listitem">
    <span className="task-name" title={task.displayName}>{task.displayName}</span>
    <span className={`task-status ${task.status}`}><StatusIcon status={task.status}/>{statusLabels[task.status]}</span>
    <span className="elapsed">{elapsed(task, now)}</span>
  </div>;
}
export function ExpandedMonitor(props: MonitorProps) {
  return <main className="monitor expanded" aria-label="Codex Monitor">
    <header onMouseDown={drag}><span className="brand">CODEX <span className="brand-secondary">MONITOR</span></span><span className={`health ${props.snapshot.quotaConnected && props.snapshot.ideConnected ? 'online' : ''}`} title={props.error ?? props.snapshot.diagnostic}/><span className="drag-fill"/></header>
    <div className="quotas"><Quotas {...props}/></div>
    <div className="task-list" role="list" aria-label="Codex threads">{props.snapshot.tasks.map(task => <TaskRow key={task.threadId} task={task} now={props.now}/>)}
      {props.snapshot.tasks.length === 0 && <div className="empty">{props.snapshot.ideConnected ? 'Waiting for a Codex turn' : 'Waiting for Codex in VS Code'}</div>}
    </div>
    <footer><span className="footer-note" title={props.snapshot.diagnostic}>{props.error ?? (!props.snapshot.quotaConnected ? 'Quota connection unavailable' : 'Local monitor · Approval visibility limited')}</span><span className="footer-controls"><button aria-label="Settings" title="Settings" onClick={()=>void invoke('open_settings')}><SettingsIcon size={14}/></button><button aria-label="Close Codex Monitor" title="Close Codex Monitor" onClick={()=>void invoke('quit_app')}><X size={15}/></button><WindowControls {...props}/></span></footer>
  </main>;
}
