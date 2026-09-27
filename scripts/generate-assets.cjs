// Original vector icon rasterized by Tauri's CLI; original gentle PCM sound presets.
const fs=require('node:fs');const path=require('node:path');
fs.mkdirSync('src-tauri/assets',{recursive:true});fs.mkdirSync('src-tauri/icons',{recursive:true});
fs.writeFileSync('src-tauri/icons/source.svg',`<svg xmlns="http://www.w3.org/2000/svg" width="256" height="256" viewBox="0 0 256 256"><rect width="256" height="256" rx="56" fill="#111820"/><path d="M92 75L42 128l50 53M164 75l50 53-50 53" fill="none" stroke="#6dd5c7" stroke-width="19" stroke-linecap="round" stroke-linejoin="round"/><path d="M145 72l-34 112" stroke="#eff5f4" stroke-width="14" stroke-linecap="round"/></svg>`);
for(const [name,frequencies] of [['needs-action',[440,392]],['done',[523.25,659.25]]]) {
  const rate=44100,duration=.42,n=Math.floor(rate*duration),data=Buffer.alloc(n*2);
  for(let i=0;i<n;i++){const t=i/rate;let v=0;for(let j=0;j<2;j++){const u=t-j*.16;if(u>=0&&u<.25){const envelope=Math.sin(Math.PI*u/.25)**2;v+=.24*envelope*(Math.sin(2*Math.PI*frequencies[j]*u)+.12*Math.sin(4*Math.PI*frequencies[j]*u));}}data.writeInt16LE(Math.round(Math.max(-1,Math.min(1,v))*32767),i*2);}
  const h=Buffer.alloc(44);h.write('RIFF');h.writeUInt32LE(36+data.length,4);h.write('WAVEfmt ',8);h.writeUInt32LE(16,16);h.writeUInt16LE(1,20);h.writeUInt16LE(1,22);h.writeUInt32LE(rate,24);h.writeUInt32LE(rate*2,28);h.writeUInt16LE(2,32);h.writeUInt16LE(16,34);h.write('data',36);h.writeUInt32LE(data.length,40);fs.writeFileSync(path.join('src-tauri/assets',name+'.wav'),Buffer.concat([h,data]));
}
