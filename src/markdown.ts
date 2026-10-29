import {marked} from 'marked';
import DOMPurify from 'dompurify';
marked.use({renderer:{html:()=>'',image:()=>''}});
export function renderMarkdown(source:string):string {
  const html=marked.parse(source,{gfm:true,breaks:true,async:false}) as string;
  return DOMPurify.sanitize(html,{ALLOWED_TAGS:['p','br','strong','em','del','blockquote','ul','ol','li','code','pre','a','h1','h2','h3','h4','h5','h6','table','thead','tbody','tr','th','td','hr'],ALLOWED_ATTR:['href','title'],ALLOW_DATA_ATTR:false,FORBID_TAGS:['img','svg','math','style','script','iframe','form']});
}
export function isSafeUrl(raw:string):boolean {
  try {const u=new URL(raw);return (u.protocol==='http:'||u.protocol==='https:') && !!u.hostname;}catch{return false;}
}
