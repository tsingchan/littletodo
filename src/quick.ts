import {invoke} from '@tauri-apps/api/core';
import {listen} from '@tauri-apps/api/event';
import {root,uid,on,showError} from './shared';
import type {Task} from './types';
export async function setupQuick(){
 root.innerHTML='<div class="window-heading"><span class="mark">✓</span><div><h1>快速记录</h1><p>记下来，让思绪轻一点。</p></div><button class="subtle show-list" id="show-list">显示待办</button><button class="icon-button" id="settings" title="设置" aria-label="设置">⚙</button></div><label class="field-label" for="content">事项内容 <span>支持 Markdown</span></label><textarea id="content" aria-label="事项内容" placeholder="负责人：&#10;要做的事……&#10;https://"></textarea><footer class="editor-footer"><label>优先级 <select id="priority" aria-label="优先级"><option>P1</option><option selected>P2</option><option>P3</option></select></label><span class="hint">Ctrl + Enter 保存</span><button class="subtle" id="cancel">关闭</button><button class="primary" id="save">保存事项</button></footer><p id="error" class="error" role="alert"></p>';
 const body=document.querySelector<HTMLTextAreaElement>('#content')!,priority=document.querySelector<HTMLSelectElement>('#priority')!,button=document.querySelector<HTMLButtonElement>('#save')!;
 try {const draft=await invoke<{body:string;priority:string}|null>('get_draft');if(draft){body.value=draft.body||'';priority.value=draft.priority||'P2';}}catch(e){showError(e);}
 let command=uid(),saving=false;
 const keep=()=>invoke('set_draft',{value:{body:body.value,priority:priority.value}});
 const dirty=(value:boolean)=>invoke('set_dirty',{dirty:value});
 const close=async()=>{if(saving)return;try{await keep();await dirty(false);await invoke('close_aux');}catch(e){showError(e);}};
 const save=async()=>{
   if(saving)return;saving=true;button.disabled=true;body.disabled=true;priority.disabled=true;document.querySelector('#error')!.textContent='';
   try {await invoke<Task>('save_task',{input:{command_id:command,id:null,expected_version:null,content_markdown:body.value,priority:priority.value}});body.value='';priority.value='P2';command=uid();await dirty(false);await invoke('close_aux');}
   catch(e){showError(e);}finally{saving=false;button.disabled=false;body.disabled=false;priority.disabled=false;}
 };
 body.addEventListener('input',()=>{command=uid();dirty(true).catch(showError);});priority.addEventListener('change',()=>{command=uid();dirty(true).catch(showError);});
 on('#save','click',()=>void save());on('#cancel','click',()=>void close());
 on('#show-list','click',()=>invoke('show_list').catch(showError));
 on('#settings','click',()=>invoke('open_settings').catch(showError));
 await listen('attempt-close',close);await listen('focus-input',()=>body.focus());await listen('quit-blocked',()=>showError('请先保存事项，或关闭窗口以保留草稿，然后再退出。'));
 document.addEventListener('keydown',e=>{if(e.key==='Enter'&&(e.ctrlKey||e.metaKey)){e.preventDefault();void save();}if(e.key==='Escape')void close();});
 body.focus();
}
