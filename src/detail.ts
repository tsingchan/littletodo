import {invoke} from '@tauri-apps/api/core';
import {listen} from '@tauri-apps/api/event';
import {root,uid,on,showError} from './shared';
import {renderMarkdown,isSafeUrl} from './markdown';
import type {Task} from './types';
export async function setupDetail(){
 root.innerHTML='<div class="detail-head"><div><span class="eyebrow">事项详情</span><h1>编辑事项</h1></div></div><div class="detail-toolbar"><label>优先级 <select id="priority"><option>P1</option><option>P2</option><option>P3</option></select></label><div class="tabs"><button id="edit-tab" class="active">编辑</button><button id="preview-tab">预览</button></div></div><textarea id="content" aria-label="事项正文"></textarea><div id="preview" class="markdown" hidden></div><div class="detail-footer"><span id="meta"></span><button id="save" class="primary">保存修改</button></div><div id="discard" class="confirm" hidden><strong>有未保存的修改</strong><p>关闭后这次修改将丢失。</p><div><button id="keep-editing">继续编辑</button><button id="discard-close">放弃修改并关闭</button></div></div><p id="error" class="error" role="alert"></p>';
 const body=document.querySelector<HTMLTextAreaElement>('#content')!,priority=document.querySelector<HTMLSelectElement>('#priority')!,preview=document.querySelector<HTMLElement>('#preview')!,saveButton=document.querySelector<HTMLButtonElement>('#save')!,dialog=document.querySelector<HTMLElement>('#discard')!;
 let task:Task;let taskId='';let savedBody='',savedPriority='',saving=false,command=uid();
 const setDirty=(value:boolean)=>invoke('set_dirty',{dirty:value});
 const dirty=()=>body.value!==savedBody||priority.value!==savedPriority;
 const load=async(id:string)=>{
   taskId=id;
   task=await invoke<Task>('get_task',{id:taskId});
   body.value=task.content_markdown;priority.value=task.priority;savedBody=body.value;savedPriority=priority.value;command=uid();
   document.querySelector('#meta')!.textContent='创建于 '+task.created_local_date;
   document.querySelector('#error')!.textContent='';
   await setDirty(false);
 };
 const openId=async(id:string)=>{if(dirty()){showError('尚未保存，请先保存');return;}await load(id);};
 try{
   const initial=await invoke<string|null>('current_detail_id');
   if(!initial){showError('未指定要编辑的事项');return;}
   await load(initial);
 }catch(e){showError(e);return;}
 const close=async()=>{if(saving)return;if(dirty()){dialog.hidden=false;return;}await invoke('close_aux');};
 const save=async()=>{if(saving)return;saving=true;saveButton.disabled=true;body.disabled=true;priority.disabled=true;
   try{task=await invoke<Task>('save_task',{input:{command_id:command,id:task.id,expected_version:task.version,content_markdown:body.value,priority:priority.value}});savedBody=body.value;savedPriority=priority.value;command=uid();document.querySelector('#error')!.textContent='';await setDirty(false);await invoke('close_aux');}
   catch(e){showError(e);}finally{saving=false;saveButton.disabled=false;body.disabled=false;priority.disabled=false;}};
 const edited=()=>{command=uid();setDirty(dirty()).catch(showError);};body.addEventListener('input',edited);priority.addEventListener('change',edited);
 on('#save','click',()=>void save());
 on('#keep-editing','click',()=>{dialog.hidden=true;});
 on('#discard-close','click',()=>{body.value=savedBody;priority.value=savedPriority;dialog.hidden=true;setDirty(false).then(()=>invoke('close_aux')).catch(showError);});
 on('#edit-tab','click',()=>{body.hidden=false;preview.hidden=true;document.querySelector('#edit-tab')!.classList.add('active');document.querySelector('#preview-tab')!.classList.remove('active');});
 on('#preview-tab','click',()=>{preview.innerHTML=renderMarkdown(body.value);body.hidden=true;preview.hidden=false;document.querySelector('#preview-tab')!.classList.add('active');document.querySelector('#edit-tab')!.classList.remove('active');});
 preview.addEventListener('click',event=>{const anchor=(event.target as HTMLElement).closest('a');if(!anchor)return;event.preventDefault();const href=anchor.getAttribute('href')||'';if(isSafeUrl(href))invoke('open_url',{url:href}).catch(showError);else showError('只允许打开 HTTP 或 HTTPS 链接');});
 await listen('attempt-close',close);
 await listen<{id:string}>('detail-open',event=>{void openId(event.payload.id);});
 await listen('detail-reopen',()=>{if(!dirty())load(taskId).catch(showError);else showError('尚未保存，请先保存');});
 await listen('detail-unsaved-block',()=>showError('尚未保存，请先保存'));
 await listen('quit-blocked',()=>{dialog.hidden=false;});
 document.addEventListener('keydown',e=>{if(e.key==='Enter'&&(e.ctrlKey||e.metaKey)){e.preventDefault();void save();}if(e.key==='Escape')void close();});
}
