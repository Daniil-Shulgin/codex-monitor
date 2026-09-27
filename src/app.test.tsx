import { act, cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { defaultSettings, emptySnapshot } from './model';
const api = vi.hoisted(()=>({invoke:vi.fn(),handlers:new Map<string,(e:{payload:unknown})=>void>(),stop:vi.fn()}));
vi.mock('@tauri-apps/api/core',()=>({invoke:api.invoke}));
vi.mock('@tauri-apps/api/event',()=>({emit:vi.fn(),listen:vi.fn(async(name:string,handler:(e:{payload:unknown})=>void)=>{api.handlers.set(name,handler);return api.stop;})}));
import App from './App';
import { SettingsPanel } from './components/SettingsPanel';
beforeEach(()=>{api.handlers.clear();api.stop.mockClear();api.invoke.mockImplementation(async(command:string)=>command==='bootstrap'?{snapshot:emptySnapshot,settings:{...defaultSettings,theme:'dark'},error:null}:undefined);});
afterEach(()=>{cleanup();window.history.replaceState({},'', '/');});
it('renders settings in the dedicated window without resizing the widget',async()=>{
  window.history.replaceState({},'', '/index.html?settings');
  render(<App/>);
  await waitFor(()=>expect(screen.getByRole('heading',{name:'Codex Monitor'})).toBeInTheDocument());
  expect(screen.getByRole('combobox',{name:'Theme'})).toBeInTheDocument();
  expect(api.invoke).not.toHaveBeenCalledWith('resize_widget',expect.anything());
});
it('closes the application from the expanded footer',async()=>{
  api.invoke.mockImplementation(async(command:string)=>command==='bootstrap'?{snapshot:emptySnapshot,settings:{...defaultSettings,expanded:true},error:null}:undefined);
  render(<App/>);
  const close=await screen.findByRole('button',{name:'Close Codex Monitor'});
  await waitFor(()=>expect(api.invoke).toHaveBeenCalledWith('resize_widget',{width:330,height:160}));
  fireEvent.click(close);
  expect(api.invoke).toHaveBeenCalledWith('quit_app');
});
it('applies persisted theme and responds to native settings changes without restart',async()=>{
  const {unmount}=render(<App/>);
  await waitFor(()=>expect(document.documentElement.dataset.theme).toBe('dark'));
  act(()=>api.handlers.get('settings')?.({payload:{...defaultSettings,theme:'light',expanded:true}}));
  await waitFor(()=>expect(document.documentElement.dataset.theme).toBe('light'));
  expect(screen.getByLabelText('Collapse')).toBeInTheDocument();
  unmount();expect(api.stop).toHaveBeenCalledTimes(3);
});
it('shows native errors and keeps missing data unavailable',async()=>{
  render(<App/>);await waitFor(()=>expect(api.handlers.has('runtime-error')).toBe(true));
  await waitFor(()=>expect(document.documentElement.dataset.theme).toBe('dark'));
  act(()=>api.handlers.get('runtime-error')?.({payload:'Shortcut conflict'}));
  expect(screen.getByTitle('Shortcut conflict')).toBeInTheDocument();
  expect(screen.getByLabelText('5H quota unavailable')).toHaveTextContent('—');
  fireEvent.mouseDown(screen.getByLabelText('Codex Monitor'),{button:0});
  expect(api.invoke).toHaveBeenCalledWith('drag_widget');
});
it('records a physical shortcut from a Russian keyboard layout',async()=>{
  const save=vi.fn(async()=>true);
  render(<SettingsPanel settings={defaultSettings} save={save} error={null}/>);
  const button=screen.getByRole('button',{name:'Ctrl+Shift+C'});
  fireEvent.click(button);
  fireEvent.keyDown(button,{key:'ь',code:'',ctrlKey:true,altKey:true});
  await waitFor(()=>expect(save).toHaveBeenCalledWith({...defaultSettings,shortcut:'Ctrl+Alt+M'}));
});
it('sizes the compact window for the visible status categories',async()=>{
  render(<App/>);
  await waitFor(()=>expect(api.invoke).toHaveBeenCalledWith('resize_widget',{width:290,height:40}));
  const tasks=(['running','needsAction','done'] as const).map((status,index)=>({threadId:String(index),turnId:'turn',displayName:'Task',status,startedAt:0,completedAt:null,durationMs:null,updatedAt:0}));
  for (const [count,width] of [[1,330],[2,360],[3,390]]) {
    act(()=>api.handlers.get('snapshot')?.({payload:{...emptySnapshot,tasks:tasks.slice(0,count)}}));
    await waitFor(()=>expect(api.invoke).toHaveBeenCalledWith('resize_widget',{width,height:40}));
  }
});
