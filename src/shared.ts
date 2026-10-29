import {invoke} from '@tauri-apps/api/core';
import {getCurrentWebviewWindow} from '@tauri-apps/api/webviewWindow';
export const root=document.querySelector<HTMLElement>('#app')!;
export const win=getCurrentWebviewWindow();
export const uid=()=>crypto.randomUUID();
export function showError(e:unknown,target='#error'){const el=document.querySelector<HTMLElement>(target);if(el)el.textContent=String(e);}
export function on(id:string,event:string,fn:EventListener){document.querySelector(id)?.addEventListener(event,fn);}
export async function openDetail(id:string){await invoke('open_detail',{id});}
