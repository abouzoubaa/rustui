const PROTOCOL="rustui.preview.postmessage",VERSION=1,MAX_BYTES=1048576;
export function requestedProfile(browser=globalThis.window){
  try {if(!browser || browser.parent===browser || browser.location.origin==="null" || browser.parent.location.origin!==browser.location.origin)return "";
    const profile=new URLSearchParams(browser.location.search).get("rustuiPreviewHost");return ["preview","composition"].includes(profile)?profile:"";
  }catch{return "";}
}
export function freshToken(browser=globalThis.window){const bytes=new Uint8Array(16);browser.crypto.getRandomValues(bytes);return Array.from(bytes,n=>n.toString(16).padStart(2,"0")).join("");}
export function createBridge(receive,disconnected=()=>{},browser=globalThis.window){
  const profile=requestedProfile(browser);if(!profile)throw new Error("reference iframe unavailable");
  const handle={browser,parent:browser.parent,origin:browser.location.origin,profile,alive:true,ready:false,session:null,disconnected};
  handle.message=event=>{if(!handle.alive||event.source!==handle.parent||event.origin!==handle.origin||typeof event.data!=="string")return;
    if(event.data.length>MAX_BYTES||new TextEncoder().encode(event.data).byteLength>MAX_BYTES)return;
    receive(event.data);
  };
  handle.pagehide=()=>closeBridge(handle);
  browser.addEventListener("message",handle.message);browser.addEventListener("pagehide",handle.pagehide);return handle;
}
export function markReady(handle){if(!handle.alive||handle.ready)return;handle.ready=true;handle.parent.postMessage(JSON.stringify({protocol:PROTOCOL,version:VERSION,event:"ready",profile:handle.profile}),handle.origin);}
export function postReply(handle,wire,session){if(!handle.alive)return;if(session)handle.session=session;handle.parent.postMessage(wire,handle.origin);}
export function closeBridge(handle){if(!handle.alive)return;handle.alive=false;handle.browser.removeEventListener("message",handle.message);handle.browser.removeEventListener("pagehide",handle.pagehide);handle.disconnected();handle.parent.postMessage(JSON.stringify({protocol:PROTOCOL,version:VERSION,event:"closed",session:handle.session}),handle.origin);}
