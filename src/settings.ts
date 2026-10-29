import {invoke} from '@tauri-apps/api/core';
import {listen} from '@tauri-apps/api/event';
import {root,on,showError} from './shared';
import type {Preferences} from './types';
export async function setupSettings(){
 let prefs=await invoke<Preferences>('get_preferences');const info=await invoke<{data_path:string;data_dir:string;shortcut_error:string|null;version:string}>('app_info');
 root.innerHTML='<div class="detail-head"><div><span class="eyebrow">LittleTodo</span><h1>设置</h1></div><button id="close" class="icon-button" aria-label="关闭">×</button></div><div class="settings-form"><label>背景颜色<select id="color"><option value="cream">奶油黄</option><option value="mint">薄荷绿</option><option value="blue">浅蓝色</option><option value="pink">柔粉色</option></select></label><label>背景不透明度 <span id="opacity-value"></span><input id="opacity" type="range" min="0.4" max="1" step="0.05"></label><label>显示待办快捷键<input id="shortcut-show" type="text" placeholder="Ctrl+Alt+L" maxlength="50"></label><label>快速记录快捷键<input id="shortcut-quick" type="text" placeholder="Ctrl+Alt+N" maxlength="50"><small>两个快捷键不能相同；若被占用请更换。</small></label><label class="checkbox-label"><input id="preview-content" type="checkbox"> 通知显示事项摘要</label><div><button id="test-notification">测试系统提醒</button><small id="notification-status"></small></div><div class="data-block"><strong>数据存储目录</strong><p id="dir" class="data-path"></p><div class="data-actions"><button id="open-data-dir" type="button">打开目录</button></div><p class="data-file"><small>数据库文件：<span id="path-file"></span></small></p><div class="agent-guide-entry"><button type="button" id="open-agent-guide">Agent 使用指南</button><small>只读打开本地库的说明与可复制提示词</small></div></div></div><p id="error" class="error" role="alert"></p><div class="detail-footer"><span id="version" class="settings-version"></span><button id="save" class="primary">保存设置</button></div>';
 const color=document.querySelector<HTMLSelectElement>('#color')!,opacity=document.querySelector<HTMLInputElement>('#opacity')!,shortcutQuick=document.querySelector<HTMLInputElement>('#shortcut-quick')!,shortcutShow=document.querySelector<HTMLInputElement>('#shortcut-show')!,preview=document.querySelector<HTMLInputElement>('#preview-content')!;
 color.value=prefs.color;opacity.value=String(prefs.opacity);shortcutQuick.value=prefs.shortcut;shortcutShow.value=prefs.shortcut_show_list||'Ctrl+Alt+L';preview.checked=prefs.notifications_preview;
 document.querySelector('#dir')!.textContent=info.data_dir;document.querySelector('#path-file')!.textContent=info.data_path;
 document.querySelector('#version')!.textContent='版本 V'+info.version;
 const update=()=>{document.querySelector('#opacity-value')!.textContent=Math.round(Number(opacity.value)*100)+'%';};opacity.addEventListener('input',update);update();if(info.shortcut_error)showError(info.shortcut_error);
 const close=()=>invoke('close_aux').catch(showError);on('#close','click',()=>void close());
 on('#open-data-dir','click',()=>invoke('open_data_dir').catch(showError));
 on('#open-agent-guide','click',()=>invoke('open_agent_guide').catch(showError));
 on('#save','click',async()=>{try{prefs={...prefs,color:color.value as Preferences['color'],opacity:Number(opacity.value),shortcut:shortcutQuick.value.trim(),shortcut_show_list:shortcutShow.value.trim(),notifications_preview:preview.checked};await invoke('save_preferences',{prefs});await invoke('close_aux');}catch(e){showError(e);}});
 on('#test-notification','click',async()=>{try{await invoke('test_notification');document.querySelector('#notification-status')!.textContent='已请求系统显示提醒，请检查系统通知设置。';}catch(e){showError(e);}});
 await listen('attempt-close',close);
}
