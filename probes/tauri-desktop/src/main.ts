import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import './styles.css';

const root = document.querySelector<HTMLElement>('#app')!;
const label = getCurrentWebviewWindow().label;
const send = (name: string) => invoke('event', { name });
function showError(error: unknown) { const target=document.querySelector('#message'); if(target) target.textContent=String(error); }
function bind(id: string, action: () => Promise<unknown>) {
  document.querySelector(id)?.addEventListener('click', () => { action().catch(showError); });
}

if (label === 'main') {
  root.innerHTML = `<h1>Tauri / WebView2 桌面探针</h1><p>合成内容 · 不存真实任务 · 不注册自启</p>
  <div class="controls"><button id="attach">挂到桌面</button><button id="detail">打开详情</button><button id="refresh">刷新事件</button><button id="quit">退出探针</button></div>
  <label>背景透明度 <input id="alpha" type="range" min="0.4" max="1" step="0.1" value="0.8"></label>
  <label><input id="blue" type="checkbox"> 使用蓝色背景</label><button id="apply">应用外观</button>
  <p id="message" role="status">先点“挂到桌面”，再显示桌面测试黄色列表。</p><pre id="events"></pre>`;
  const refresh=async()=>{const rows=await invoke<string[]>('status');document.querySelector('#events')!.textContent=rows.slice(-18).join('\n');};
  bind('#attach',async()=>{const result=await invoke<string>('attach');document.querySelector('#message')!.textContent=result;await refresh();});
  bind('#detail',()=>invoke('open_detail'));
  bind('#refresh',refresh);
  bind('#apply',async()=>{await invoke('appearance',{alpha:Number(document.querySelector<HTMLInputElement>('#alpha')!.value),blue:document.querySelector<HTMLInputElement>('#blue')!.checked});await refresh();});
  bind('#quit',()=>invoke('quit'));
  await refresh();
} else if (label === 'panel') {
  document.documentElement.classList.add('desktop-root');
  document.body.classList.add('desktop');
  root.innerHTML=`<section class="card"><header id="drag">LittleTodo <span>拖动这里</span></header><label class="task"><input id="task" type="checkbox"><b>P1</b><span>负责人：示例 · 验证桌面交互</span><button id="detail" aria-label="编辑事项">✎</button></label><div class="task"><b>P2</b><span class="summary">超长事项的前部摘要：中文输入、URL、列表滚动与详情窗口</span></div><p>背景应能透出桌面。普通应用可遮挡此面板。</p><p id="message" role="status"></p></section>`;
  document.querySelector<HTMLInputElement>('#task')!.addEventListener('change',(e)=>{send((e.target as HTMLInputElement).checked?'checkbox_on':'checkbox_off').catch(showError);});
  bind('#detail',()=>invoke('open_detail'));
  await listen<{alpha:number;blue:boolean}>('appearance',async({payload})=>{
    root.style.setProperty('--card-bg',payload.blue?`rgba(194,222,246,${payload.alpha})`:`rgba(247,228,161,${payload.alpha})`);
    await send('appearance_applied');
  });
  const header=document.querySelector<HTMLElement>('#drag')!;
  let drag: {sx:number;sy:number;x:number;y:number}|null=null;
  header.addEventListener('pointerdown',async e=>{
    if(e.button!==0)return;
    header.setPointerCapture(e.pointerId);
    const pos=await getCurrentWebviewWindow().outerPosition();
    drag={sx:e.screenX,sy:e.screenY,x:pos.x,y:pos.y};
  });
  header.addEventListener('pointermove',e=>{if(drag)invoke('move_panel',{x:Math.round(drag.x+e.screenX-drag.sx),y:Math.round(drag.y+e.screenY-drag.sy)}).catch(showError);});
  header.addEventListener('pointerup',()=>{if(drag){drag=null;send('drag_end').catch(showError);}});
} else {
  root.innerHTML=`<h1>事项详情 · 输入法测试</h1><textarea id="content" aria-label="事项Markdown正文">负责人：示例\n- [ ] 测试中文编辑\nhttps://example.com</textarea><p>纯输入探针，不保存数据、不访问链接。</p><p id="message" role="status"></p>`;
  document.querySelector('#content')!.addEventListener('change',()=>{send('detail_edited').catch(showError);});
}
await send('ready');
