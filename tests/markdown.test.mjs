import { test } from 'node:test';
import assert from 'node:assert/strict';
import { JSDOM } from 'jsdom';
const dom = new JSDOM('<!doctype html><html><body></body></html>');
globalThis.window=dom.window;
globalThis.document=dom.window.document;
const {renderMarkdown,isSafeUrl}=await import('../src/markdown.ts');
const {plainSummary}=await import('../src/summary.ts');

test('Markdown preserves ordinary formatting and blocks raw HTML and remote images',()=>{
  const html=renderMarkdown('**重点**\n\n<script>alert(1)</script>\n\n![x](https://example.com/pixel)\n\n[文档](https://example.com/a)');
  assert.match(html,/<strong>重点<\/strong>/);
  assert.doesNotMatch(html,/<script|<img|alert\(1\)/i);
  assert.match(html,/href="https:\/\/example\.com\/a"/);
});
test('unsafe link schemes never pass the open-link gate',()=>{
  for(const value of ['javascript:alert(1)','file:///C:/test','data:text/html,test','mailto:a@example.com','/relative'])assert.equal(isSafeUrl(value),false,value);
  assert.equal(isSafeUrl('https://example.com/a'),true);
  assert.doesNotMatch(renderMarkdown('[bad](javascript:alert%281%29)'),/href="javascript:/i);
});
test('list summary keeps link label and flattens lines',()=>{
  assert.equal(plainSummary('# 标题\n- [ ] [文档](https://example.com)'), '标题 文档');
});
