import { useState, type KeyboardEvent } from 'react';
import type { Settings } from '../model';
import { Support } from './Support';
interface Props {settings:Settings;save:(s:Settings)=>Promise<boolean>;error:string|null}
const russianKeys = 'йцукенгшщзхъфывапролджэячсмитьбю';
const physicalKeys = "qwertyuiop[]asdfghjkl;'zxcvbnm,.";
export function SettingsPanel({settings,save,error}: Props) {
  const [recording,setRecording] = useState(false);
  const [busy,setBusy] = useState(false);
  const [opacity,setOpacity] = useState(settings.opacity);
  const [volume,setVolume] = useState(settings.volume);
  async function update(patch:Partial<Settings>) {setBusy(true);await save({...settings,...patch});setBusy(false);}
  function record(e:KeyboardEvent<HTMLButtonElement>) {
    if (!recording) return;
    e.preventDefault();e.stopPropagation();
    if(e.key === 'Escape') {setRecording(false);return;}
    if(['Control','Shift','Alt','Meta'].includes(e.key)) return;
    if (!e.ctrlKey && !e.altKey && !e.metaKey) return;
    const russianIndex = russianKeys.indexOf(e.key.toLowerCase());
    const code = e.code || (russianIndex >= 0 ? physicalKeys[russianIndex] : e.key);
    const key = code.startsWith('Key') ? code.slice(3) : code.startsWith('Digit') ? code.slice(5) : code;
    if (!key || key === 'Unidentified' || !/^[\x20-\x7e]+$/.test(key)) return;
    const mainKey = /^[a-z]$/.test(key) ? key.toUpperCase() : key;
    const shortcut=[e.ctrlKey?'Ctrl':'',e.shiftKey?'Shift':'',e.altKey?'Alt':'',e.metaKey?'Super':'',mainKey].filter(Boolean).join('+');
    setRecording(false);void update({shortcut});
  }
  const toggle = (key:'pinned'|'clickThrough'|'expanded'|'launchAtLogin'|'notifyAction'|'notifyDone'|'sound',label:string) => <label className="setting-row"><span>{label}</span><input type="checkbox" checked={settings[key]} disabled={busy} onChange={e=>void update({[key]:e.target.checked})}/></label>;
  return <main className="settings-panel"><h1>Codex Monitor</h1><p className="settings-subtitle">A quiet view of your local work.</p>
    <fieldset disabled={busy}><legend>Appearance</legend>
      <label className="setting-row"><span>Theme</span><select aria-label="Theme" value={settings.theme} onChange={e=>void update({theme:e.target.value as Settings['theme']})}><option value="system">System</option><option value="dark">Dark</option><option value="light">Light</option></select></label>
      <label className="setting-row"><span>Opacity <small>{Math.round(opacity*100)}%</small></span><input aria-label="Opacity" type="range" min="55" max="100" value={Math.round(opacity*100)} onChange={e=>setOpacity(Number(e.target.value)/100)} onPointerUp={()=>void update({opacity})} onKeyUp={()=>void update({opacity})}/></label>
    </fieldset>
    <fieldset><legend>Window</legend>{toggle('pinned','Always on Top')}{toggle('clickThrough','Click-through')}{toggle('expanded','Expanded view')}{toggle('launchAtLogin','Start with Windows')}</fieldset>
    <fieldset><legend>Global shortcut</legend><div className="setting-row"><span>Toggle click-through</span><button className={`shortcut ${recording?'recording':''}`} disabled={busy} onClick={()=>setRecording(true)} onKeyDown={record} onBlur={()=>setRecording(false)}>{recording?'Press shortcut…':settings.shortcut}</button></div><p className="hint">Use Ctrl, Alt or Win with a key. Escape cancels. The tray menu always restores mouse input.</p></fieldset>
    <fieldset><legend>Sounds</legend>{toggle('notifyAction','Needs action sound')}{toggle('notifyDone','Done sound')}{toggle('sound','Enable sound')}
      <label className="setting-row"><span>Volume <small>{volume}%</small></span><input aria-label="Volume" type="range" min="0" max="100" value={volume} onChange={e=>setVolume(Number(e.target.value))} onPointerUp={()=>void update({volume})} onKeyUp={()=>void update({volume})}/></label>
    </fieldset>
    <fieldset disabled={busy}><legend>Data</legend><label className="setting-row"><span>Quota refresh</span><select aria-label="Quota refresh" value={settings.refreshSeconds} onChange={e=>void update({refreshSeconds:Number(e.target.value)})}>{[10,30,60].map(n=><option key={n} value={n}>{n} sec</option>)}</select></label></fieldset>
    {error && <p className="error" role="alert">{error}</p>}<Support/>
  </main>;
}
