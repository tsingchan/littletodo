import {invoke} from '@tauri-apps/api/core';
import {listen} from '@tauri-apps/api/event';
import {root,on,showError} from './shared';
import {renderMarkdown} from './markdown';
export async function setupAgentGuide(){
 let body='';
 try{body=await invoke<string>('get_agent_readonly_doc');}catch(e){showError(e);return;}
 root.innerHTML='<div class="detail-head"><div><span class="eyebrow">LittleTodo</span><h1>Agent 使用指南</h1></div><button id="close" class="icon-button" aria-label="关闭">×</button></div><p class="agent-guide-lead">以下内容与仓库 <code>docs/AGENT-READONLY.md</code> 一致，供只读分析本地待办数据时参考。</p><div id="content" class="markdown agent-guide-body"></div><p id="error" class="error" role="alert"></p>';
 document.querySelector('#content')!.innerHTML=renderMarkdown(body);
 const close=()=>invoke('close_aux').catch(showError);
 on('#close','click',()=>void close());
 await listen('attempt-close',close);
}
