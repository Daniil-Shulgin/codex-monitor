import { ChevronDown, ChevronUp, Pin, PinOff } from 'lucide-react';
import type { Settings } from '../model';
export function WindowControls({settings,save}: {settings:Settings;save:(s:Settings)=>Promise<boolean>}) {
  const PinIcon = settings.pinned ? Pin : PinOff;
  const Chevron = settings.expanded ? ChevronDown : ChevronUp;
  return <div className="controls">
    <button title={settings.pinned ? 'Unpin · Always on Top' : 'Pin · Always on Top'} aria-label="Always on Top" aria-pressed={settings.pinned} onClick={() => void save({...settings,pinned:!settings.pinned})}><PinIcon size={14}/></button>
    <button title={settings.expanded ? 'Collapse' : 'Expand'} aria-label={settings.expanded ? 'Collapse' : 'Expand'} onClick={() => void save({...settings,expanded:!settings.expanded})}><Chevron size={16}/></button>
  </div>;
}
