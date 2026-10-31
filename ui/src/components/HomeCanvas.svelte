<script lang="ts">
  import { homeDoc, type HomeDoc } from "../lib/home";

  function renderDocument(doc: HomeDoc): string {
    // The Rust side is expected to sanitize the HTML before it reaches this
    // store. The sandbox and CSP here are the second line of defence.
    return `<!doctype html>
<html lang="zh-CN">
<head>
<meta charset="utf-8">
<meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'unsafe-inline'; img-src data:">
<meta name="viewport" content="width=device-width, initial-scale=1">
<style>
html,body{margin:0;min-height:100%;background:#fff;color:#111;font-family:"Helvetica Neue",Helvetica,Arial,"PingFang SC","Microsoft YaHei",sans-serif;overflow:hidden}
*{box-sizing:border-box}
${doc.css}
</style>
</head>
<body>${doc.html}</body>
</html>`;
  }

  $: srcDoc = $homeDoc ? renderDocument($homeDoc) : "";
</script>

<div class="canvas">
  {#if srcDoc}
    <iframe
      title="服务器自定义主页"
      sandbox=""
      referrerpolicy="no-referrer"
      srcdoc={srcDoc}
    ></iframe>
  {/if}
</div>

<style>
  .canvas {
    width: 100%;
    height: 100%;
    overflow: hidden;
    background: var(--paper);
  }

  iframe {
    display: block;
    width: 100%;
    height: 100%;
    border: 0;
    background: var(--paper);
  }
</style>
