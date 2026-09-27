import { useEffect, useState, type CSSProperties } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useMonitor } from './useMonitor';
import { CollapsedMonitor, ExpandedMonitor } from './components/Monitor';
import { SettingsPanel } from './components/SettingsPanel';
const compactWidths = [290, 330, 360, 390] as const;
export default function App() {
  const {snapshot,settings,error,ready,save} = useMonitor();
  const [now,setNow] = useState(Date.now());
  const isSettings = new URLSearchParams(window.location.search).has('settings');
  const visibleStatuses=new Set(snapshot.tasks.map(task=>task.status)).size;
  useEffect(()=>{const id=setInterval(()=>setNow(Date.now()),1000);return()=>clearInterval(id);},[]);
  useEffect(()=>{document.documentElement.dataset.theme=settings.theme;document.documentElement.dataset.window=isSettings?'settings':'widget';},[settings.theme,isSettings]);
  useEffect(()=>{
    if (isSettings || !ready) return;
    const width=settings.expanded ? 330 : (compactWidths[visibleStatuses] ?? 390);
    // Header 39 + quotas 35 + list padding 8 + footer 40 + outer border 2.
    const height=settings.expanded ? 124+Math.min(Math.max(snapshot.tasks.length,1),7)*36 : 40;
    void invoke('resize_widget',{width,height}).catch(()=>{});
  },[settings.expanded,snapshot.tasks.length,visibleStatuses,isSettings,ready]);
  const props={snapshot,settings,error,now,save};
  return <div style={{'--surface-opacity':settings.opacity} as CSSProperties}>
    {isSettings ? <SettingsPanel key={ready?'ready':'loading'} settings={settings} save={save} error={error}/> : settings.expanded ? <ExpandedMonitor {...props}/> : <CollapsedMonitor {...props}/>}
  </div>;
}
