import { CircleCheck, LoaderCircle, TriangleAlert } from 'lucide-react';
import type { Task, TaskStatus } from '../model';
export const statusLabels = {running:'Running',needsAction:'Needs action',done:'Done'};
const icons = {running:LoaderCircle,needsAction:TriangleAlert,done:CircleCheck};
export function StatusIcon({status}: {status:TaskStatus}) {const Icon = icons[status];return <Icon size={14} strokeWidth={1.8} aria-hidden className={`status-icon ${status}`} />;}
export function StatusCounters({tasks}: {tasks:Task[]}) {
  return <>{(['running','needsAction','done'] as const).map(status => {
    const count = tasks.filter(t => t.status === status).length;
    return count ? <span className="counter" key={status} aria-label={`${statusLabels[status]}: ${count}`} title={`${statusLabels[status]}: ${count}`}><StatusIcon status={status}/><span>{count}</span></span> : null;
  })}</>;
}
