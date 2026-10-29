export function plainSummary(raw:string):string {
  return raw.replace(/!\[([^\]]*)\]\([^)]*\)/g,'').replace(/\[([^\]]+)\]\([^)]*\)/g,'$1').replace(/^\s*[-*+]\s*\[[ xX]\]\s*/gm,'').replace(/^\s{0,3}#{1,6}\s+/gm,'').replace(/[\x60*_~>]/g,'').replace(/\s+/g,' ').trim();
}
