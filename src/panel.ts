import {invoke} from '@tauri-apps/api/core';
import {listen} from '@tauri-apps/api/event';
import {root,win,uid,on,showError,openDetail} from './shared';
import {plainSummary} from './summary';
import type {Task,Preferences} from './types';
const colors:Record<Preferences['color'],string>={cream:'250,244,224',mint:'226,243,234',blue:'226,238,249',pink:'250,232,231'};
function todayLocal(){const d=new Date();return `${d.getFullYear()}-${String(d.getMonth()+1).padStart(2,'0')}-${String(d.getDate()).padStart(2,'0')}`;}
function shiftLocalDate(iso:string,delta:number){const [y,m,day]=iso.split('-').map(Number);const d=new Date(y,m-1,day+delta);return `${d.getFullYear()}-${String(d.getMonth()+1).padStart(2,'0')}-${String(d.getDate()).padStart(2,'0')}`;}
function formatDayLabel(iso:string){const [y,m,day]=iso.split('-').map(Number);return new Intl.DateTimeFormat('zh-CN',{month:'long',day:'numeric',weekday:'short'}).format(new Date(y,m-1,day));}
export async function setupPanel(){
 document.documentElement.classList.add('panel-root');document.body.classList.add('panel-body');
 root.innerHTML='<div class="panel"><header class="panel-header" id="panel-drag"><div class="brand"><span class="mark">✓</span><div><strong>LittleTodo</strong></div></div><div class="date-nav"><button id="day-prev" type="button" class="icon-button day-step" aria-label="前一天">‹</button><span id="date" class="date-label"></span><button id="day-next" type="button" class="icon-button day-step" aria-label="后一天">›</button></div><div class="header-actions"><button id="add" class="icon-button" title="快速记录" aria-label="快速记录">＋</button><button id="settings" class="icon-button" title="设置" aria-label="设置">⚙</button><button id="hide" class="icon-button" title="收起到托盘" aria-label="收起">−</button></div></header><section class="list-top"><span>待办事项</span><span class="count" id="count">0</span></section><div id="scroll" class="scroll"><div id="open-list" class="task-list"></div><div id="empty" class="empty"><span>✓</span><strong id="empty-title">今天有点轻松</strong><small id="empty-hint">点击右上角 ＋，记下一件事</small></div><details id="done-section" class="done"><summary>已完成 <span id="done-count">0</span></summary><div id="done-list" class="task-list"></div></details></div><p id="next-reminder" class="panel-reminder"></p><div class="panel-footer"><span id="hint">Ctrl + Alt + N 快速记录</span><button id="quit" class="quiet">退出</button></div><div id="error" class="panel-error" role="alert"></div><div id="copy-confirm" class="panel-confirm" hidden role="dialog" aria-modal="true" aria-labelledby="copy-confirm-title"><div class="panel-confirm-box"><strong id="copy-confirm-title">复制到今日</strong><p>今天已有相同事项，是否再复制一份？</p><div class="panel-confirm-actions"><button type="button" id="copy-cancel" class="subtle">取消</button><button type="button" id="copy-ok" class="primary">仍要复制</button></div></div></div><span class="panel-resize panel-resize-e" aria-hidden="true"></span><span class="panel-resize panel-resize-s" aria-hidden="true"></span><span class="panel-resize panel-resize-se" title="拖动放大窗口" aria-label="拖动放大窗口"></span></div>';
 let prefs=await invoke<Preferences>('get_preferences');let tasks:Task[]=[];let dragging:string|null=null;
 let viewDate=todayLocal();const today=todayLocal();
 const copyDialog=document.querySelector<HTMLElement>('#copy-confirm')!;
 let copyResolve:((ok:boolean)=>void)|null=null;
 function askCopyDuplicate(){return new Promise<boolean>(resolve=>{copyResolve=resolve;copyDialog.hidden=false;document.querySelector<HTMLButtonElement>('#copy-ok')!.focus();});}
 function closeCopyDialog(ok:boolean){copyDialog.hidden=true;if(copyResolve){copyResolve(ok);copyResolve=null;}}
 on('#copy-cancel','click',()=>closeCopyDialog(false));
 on('#copy-ok','click',()=>closeCopyDialog(true));
 function appearance(p:Preferences){prefs=p;root.style.setProperty('--paper','rgba('+(colors[p.color]??colors.cream)+','+p.opacity+')');document.querySelector('#hint')!.textContent=p.shortcut_show_list+' 显示待办 · '+p.shortcut+' 快速记录';}
 async function updateNextReminder(){try{const info=await invoke<{label:string}>('next_reminder_info');document.querySelector('#next-reminder')!.textContent=info.label;}catch{document.querySelector('#next-reminder')!.textContent='';}}
 function updateDayNav(){
   document.querySelector('#date')!.textContent=formatDayLabel(viewDate);
   const next=document.querySelector<HTMLButtonElement>('#day-next')!;
   next.disabled=viewDate>=today;
   const emptyTitle=document.querySelector('#empty-title')!;
   const emptyHint=document.querySelector('#empty-hint')!;
   if(viewDate===today){emptyTitle.textContent='今天有点轻松';emptyHint.textContent='点击右上角 ＋，记下一件事';}
   else{emptyTitle.textContent='这一天没有事项';emptyHint.textContent='可用右侧复制按钮把历史事项加到今日';}
 }
 appearance(prefs);updateDayNav();
 const refresh=async()=>{try{tasks=await invoke<Task[]>('list_tasks',{localDate:viewDate});draw();await updateNextReminder();}catch(e){showError(e);}};
 function draw(){
   const open=tasks.filter(t=>t.status==='open'),done=tasks.filter(t=>t.status==='completed');
   const openList=document.querySelector('#open-list')!,doneList=document.querySelector('#done-list')!;
   openList.replaceChildren();doneList.replaceChildren();
   document.querySelector('#count')!.textContent=String(open.length);document.querySelector('#done-count')!.textContent=String(done.length);
   (document.querySelector('#empty') as HTMLElement).hidden=open.length>0 || done.length>0;
   const section=document.querySelector<HTMLDetailsElement>('#done-section')!;section.hidden=done.length===0;
   for(const task of tasks) (task.status==='open'?openList:doneList).append(row(task));
 }
 function row(task:Task):HTMLElement {
   const item=document.createElement('div');item.className='task-row'+(task.status==='completed'?' completed':'');item.dataset.id=task.id;
   if(task.status==='open'){
     item.draggable=true;
     item.addEventListener('dragstart',e=>{dragging=task.id;item.classList.add('dragging');e.dataTransfer?.setData('text/plain',task.id);if(e.dataTransfer)e.dataTransfer.effectAllowed='move';});
     item.addEventListener('dragend',()=>{dragging=null;item.classList.remove('dragging');});
     item.addEventListener('dragover',e=>{if(dragging && dragging!==task.id){e.preventDefault();item.classList.add('drop-target');}});
     item.addEventListener('dragleave',()=>item.classList.remove('drop-target'));
     item.addEventListener('drop',e=>{e.preventDefault();item.classList.remove('drop-target');void reorder(dragging,task.id);});
   }
   const check=document.createElement('input');check.type='checkbox';check.checked=task.status==='completed';check.className='check';check.setAttribute('aria-label',check.checked?'取消完成':'完成事项');
   check.addEventListener('change',async()=>{check.disabled=true;try{await invoke('complete_task',{input:{command_id:uid(),id:task.id,expected_version:task.version,completed:check.checked}});}catch(e){showError(e);await refresh();}finally{check.disabled=false;}});
   const priority=document.createElement('span');priority.className='priority '+task.priority.toLowerCase();priority.textContent=task.priority;
   const text=document.createElement('button');text.className='task-text';text.title=plainSummary(task.content_markdown);text.textContent=plainSummary(task.content_markdown)||'（空白事项）';text.addEventListener('click',()=>openDetail(task.id).catch(showError));
   const actions=document.createElement('div');actions.className='row-actions';
   const edit=document.createElement('button');edit.className='row-action';edit.title='编辑';edit.setAttribute('aria-label','编辑事项');edit.textContent='✎';edit.addEventListener('click',()=>openDetail(task.id).catch(showError));
   const copy=document.createElement('button');copy.className='row-action';copy.title='复制到今日';copy.setAttribute('aria-label','复制到今日');copy.textContent='⎘';copy.addEventListener('click',async()=>{copy.disabled=true;try{
     const todayTasks=await invoke<Task[]>('list_tasks',{localDate:today});
     const dup=todayTasks.some(t=>t.content_markdown===task.content_markdown&&t.priority===task.priority);
     if(dup&&!await askCopyDuplicate())return;
     await invoke('copy_task_to_today',{input:{command_id:uid(),source_id:task.id}});
     if(viewDate===today)await refresh();
   }catch(e){showError(e);}finally{copy.disabled=false;}});
   actions.append(edit,copy);
   item.append(check,priority,text,actions);return item;
 }
 async function reorder(source:string|null,target:string){
   if(!source||source===target)return;
   const open=tasks.filter(t=>t.status==='open');const from=open.findIndex(t=>t.id===source),to=open.findIndex(t=>t.id===target);
   if(from<0||to<0)return;
   const [item]=open.splice(from,1);open.splice(to,0,item);
   try{await invoke('reorder_tasks',{input:{command_id:uid(),local_date:viewDate,tasks:open.map(t=>({id:t.id,version:t.version}))}});}catch(e){showError(e);await refresh();}
 }
 const scroller=document.querySelector<HTMLElement>('#scroll')!;
 scroller.addEventListener('dragover',e=>{if(!dragging)return;const rect=scroller.getBoundingClientRect();if(e.clientY<rect.top+35)scroller.scrollTop-=16;else if(e.clientY>rect.bottom-35)scroller.scrollTop+=16;});
 on('#day-prev','click',()=>{viewDate=shiftLocalDate(viewDate,-1);updateDayNav();void refresh();});
 on('#day-next','click',()=>{if(viewDate>=today)return;viewDate=shiftLocalDate(viewDate,1);updateDayNav();void refresh();});
 on('#add','click',()=>invoke('new_task').catch(showError));on('#settings','click',()=>invoke('open_settings').catch(showError));on('#hide','click',()=>invoke('hide_panel').catch(showError));on('#quit','click',()=>invoke('quit_app').catch(showError));
 await listen('tasks-changed',refresh);await listen('refresh-tasks',refresh);await listen<Preferences>('preferences-changed',event=>appearance(event.payload));await listen('desktop-error',event=>showError(event.payload));
 const info=await invoke<{shortcut_error:string|null;tray_error:string|null}>('app_info');if(info.shortcut_error||info.tray_error)showError(info.shortcut_error||info.tray_error);
 const header=document.querySelector<HTMLElement>('#panel-drag')!;
 let down=false,drag:{sx:number;sy:number;x:number;y:number}|null=null,last:{x:number;y:number}|null=null,inflight=false;
 header.addEventListener('pointerdown',async e=>{if(e.button!==0 || (e.target as HTMLElement).closest('button'))return;down=true;header.setPointerCapture(e.pointerId);const pos=await win.outerPosition();if(down)drag={sx:e.screenX,sy:e.screenY,x:pos.x,y:pos.y};});
 header.addEventListener('pointermove',e=>{if(!down||!drag)return;last={x:Math.round(drag.x+(e.screenX-drag.sx)*window.devicePixelRatio),y:Math.round(drag.y+(e.screenY-drag.sy)*window.devicePixelRatio)};if(!inflight){inflight=true;invoke('move_panel',last).catch(()=>{}).finally(()=>{inflight=false;});}});
 header.addEventListener('pointerup',async()=>{down=false;drag=null;if(last){try{await invoke('move_panel',last);await invoke('save_position',last);}catch(e){showError(e);}last=null;}});
 for(const [sel,dir] of [['.panel-resize-e','East'],['.panel-resize-s','South'],['.panel-resize-se','SouthEast']] as const){
  document.querySelector<HTMLElement>(sel)!.addEventListener('pointerdown',e=>{if(e.button!==0)return;e.preventDefault();e.stopPropagation();void win.startResizeDragging(dir).catch(showError);});
 }
 try{await invoke('prepare_panel');}catch(e){showError('无法初始化列表窗口：'+String(e));}
 await refresh();
 setInterval(()=>{void updateNextReminder();},60_000);
}
