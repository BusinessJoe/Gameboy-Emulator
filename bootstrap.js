(()=>{var e,t,r={},o={};function n(e){if(o[e])return o[e].exports;var t=o[e]={id:e,loaded:!1,exports:{}};return r[e](t,t.exports,n),t.loaded=!0,t.exports}n.m=r,n.n=e=>{var t=e&&e.__esModule?()=>e.default:()=>e;return n.d(t,{a:t}),t},n.d=(e,t)=>{for(var r in t)n.o(t,r)&&!n.o(e,r)&&Object.defineProperty(e,r,{enumerable:!0,get:t[r]})},n.f={},n.e=e=>Promise.all(Object.keys(n.f).reduce(((t,r)=>(n.f[r](e,t),t)),[])),n.u=e=>e+".bootstrap.js",n.g=function(){if("object"==typeof globalThis)return globalThis;try{return this||new Function("return this")()}catch(e){if("object"==typeof window)return window}}(),n.hmd=e=>((e=Object.create(e)).children||(e.children=[]),Object.defineProperty(e,"exports",{enumerable:!0,set:()=>{throw new Error("ES Modules may not assign module.exports or exports.*, Use ESM export syntax, instead: "+e.id)}}),e),n.o=(e,t)=>Object.prototype.hasOwnProperty.call(e,t),e={},t="gameboy-emulator-frontend:",n.l=(r,o,a)=>{if(e[r])e[r].push(o);else{var i,s;if(void 0!==a)for(var l=document.getElementsByTagName("script"),u=0;u<l.length;u++){var c=l[u];if(c.getAttribute("src")==r||c.getAttribute("data-webpack")==t+a){i=c;break}}i||(s=!0,(i=document.createElement("script")).charset="utf-8",i.timeout=120,n.nc&&i.setAttribute("nonce",n.nc),i.setAttribute("data-webpack",t+a),i.src=r),e[r]=[o];var d=(t,o)=>{i.onerror=i.onload=null,clearTimeout(p);var n=e[r];if(delete e[r],i.parentNode&&i.parentNode.removeChild(i),n&&n.forEach((e=>e(o))),t)return t(o)},p=setTimeout(d.bind(null,void 0,{type:"timeout",target:i}),12e4);i.onerror=d.bind(null,i.onerror),i.onload=d.bind(null,i.onload),s&&document.head.appendChild(i)}},n.r=e=>{"undefined"!=typeof Symbol&&Symbol.toStringTag&&Object.defineProperty(e,Symbol.toStringTag,{value:"Module"}),Object.defineProperty(e,"__esModule",{value:!0})},(()=>{var e;n.g.importScripts&&(e=n.g.location+"");var t=n.g.document;if(!e&&t&&(t.currentScript&&(e=t.currentScript.src),!e)){var r=t.getElementsByTagName("script");r.length&&(e=r[r.length-1].src)}if(!e)throw new Error("Automatic publicPath is not supported in this browser");e=e.replace(/#.*$/,"").replace(/\?.*$/,"").replace(/\/[^\/]+$/,"/"),n.p=e})(),(()=>{var e={179:0};n.f.j=(t,r)=>{var o=n.o(e,t)?e[t]:void 0;if(0!==o)if(o)r.push(o[2]);else{var a=new Promise(((r,n)=>{o=e[t]=[r,n]}));r.push(o[2]=a);var i=n.p+n.u(t),s=new Error;n.l(i,(r=>{if(n.o(e,t)&&(0!==(o=e[t])&&(e[t]=void 0),o)){var a=r&&("load"===r.type?"missing":r.type),i=r&&r.target&&r.target.src;s.message="Loading chunk "+t+" failed.\n("+a+": "+i+")",s.name="ChunkLoadError",s.type=a,s.request=i,o[1](s)}}),"chunk-"+t)}};var t=self.webpackChunkgameboy_emulator_frontend=self.webpackChunkgameboy_emulator_frontend||[],r=t.push.bind(t);t.push=t=>{for(var o,a,[i,s,l]=t,u=0,c=[];u<i.length;u++)a=i[u],n.o(e,a)&&e[a]&&c.push(e[a][0]),e[a]=0;for(o in s)n.o(s,o)&&(n.m[o]=s[o]);for(l&&l(n),r(t);c.length;)c.shift()()}})(),n.v=(e,t,r,o)=>{var a=fetch(n.p+""+r+".module.wasm");return"function"==typeof WebAssembly.instantiateStreaming?WebAssembly.instantiateStreaming(a,o).then((t=>Object.assign(e,t.instance.exports))):a.then((e=>e.arrayBuffer())).then((e=>WebAssembly.instantiate(e,o))).then((t=>Object.assign(e,t.instance.exports)))},n.e(10).then(n.bind(n,10)).catch((e=>console.error("Error importing `index.js`:",e)))})();