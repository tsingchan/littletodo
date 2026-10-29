# Markdown 与链接合同

状态：项目约束已明确，解析器与版本待定。来源：用户要求Markdown且无图片；CommonMark规范索引 https://spec.commonmark.org/ ，本次未联网核实。
建议支持段落、列表、强调、行内代码、代码块、http/https链接；表格/任务列表等扩展待原型确认。
原始Markdown是保存真相源；摘要派生为安全纯文本。禁原始HTML、脚本、图片及远程加载，预览不发网络请求；外链只有用户点击才经OS打开。拒绝javascript/data/file等协议，不能只靠Markdown解析器默认值。
不自动抓URL标题、不加载远程favicon。通知和列表摘要使用纯文本，不把正文当HTML。
验证合成恶意链接、嵌套语法、超长内容、中文/emoji、HTML、图片和代码块；实际解析器接口/错误处理/版本在选型后补齐。
