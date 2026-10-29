import {win,showError} from './shared';
import './styles.css';
async function start(){
 if(win.label==='main'){await (await import('./panel')).setupPanel();}
 else if(win.label==='quick'){await (await import('./quick')).setupQuick();}
 else if(win.label==='settings'){await (await import('./settings')).setupSettings();}
 else if(win.label==='detail'){await (await import('./detail')).setupDetail();}
 else if(win.label==='agent-guide'){await (await import('./agent-guide')).setupAgentGuide();}
 else {document.body.textContent='未知窗口：'+win.label;}
}
start().catch(e=>{document.body.textContent='LittleTodo 启动失败：'+String(e);showError(e);});
