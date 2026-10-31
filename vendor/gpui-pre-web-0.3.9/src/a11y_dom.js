function attribute(element,name,value) {value=String(value);if(element.getAttribute(name)!==value) element.setAttribute(name,value);}
// Semantic projection only. GPUI owns layout, focus, value, selection and actions.
// Never interpret application text as HTML and never copy a protected value here.
export class GpuiImeHost {
  constructor(document, body, touch) {
    this.document = document;
    this.host = document.createElement("div");
    this.host.setAttribute("data-gpui-ime-host", "");
    Object.assign(this.host.style, {position:"fixed",top:"0",left:"0",width:"1px",height:"1px",opacity:"0"});
    this.attributes = new Map();
    if (touch) this.attributes.set("inputmode", "none");
    this.secure = false;
 this.ownerActive = false;
    this.alive = true;
    this.readOnly = false;
    this.spellcheck = false;
    body.appendChild(this.host);
    this.replaceOwner();
    this.focus();
  }
  replaceOwner() {
    if (this.current) {
      this.current.value = "";
      this.current.remove();
    }
    const input = this.document.createElement(this.secure ? "input" : "textarea");
 input.setAttribute("aria-hidden","true");
 input.setAttribute("tabindex","-1");
 input.disabled = !this.ownerActive;
    if (this.secure) input.setAttribute("type", "password");
    Object.assign(input.style, {width:"1px",height:"1px",fontSize:"16px",padding:"0",border:"0"});
    for (const [name,value] of this.attributes) input.setAttribute(name,value);
    input.readOnly = this.readOnly;
    input.spellcheck = this.secure ? false : this.spellcheck;
    if (this.secure) {
      input.setAttribute("autocorrect","off");
      input.setAttribute("autocapitalize","off");
      input.setAttribute("autocomplete","off");
    }
    this.current = input;
    this.host.appendChild(input);
  }
  ownerChanged() { if (this.alive) {this.ownerActive=false;this.replaceOwner();} }
 setOwnerActive(active) {
  this.ownerActive=!!active;
  this.current.disabled=!this.ownerActive;
  if(!this.ownerActive) {
   this.current.value="";
   if(this.isFocused()) this.blur();
   this.current.setAttribute("aria-hidden","true");
   this.current.setAttribute("tabindex","-1");
  }
 }
 hasOwner() {return this.alive && this.ownerActive;}
  setSecure(secure) { if (this.secure !== secure) {this.secure=secure;this.replaceOwner();} }
  isSecure() { return this.secure; }
  activeElement() { return this.current; }
  setSemanticBounds(bounds) {
    if(!bounds) return;
    const [x,y,width,height]=bounds;
    Object.assign(this.host.style,{left:x+"px",top:y+"px",width:width+"px",height:height+"px",pointerEvents:"none"});
    Object.assign(this.current.style,{width:"100%",height:"100%"});
  }
  eventTarget() { return this.host; }
  containsTarget(target) { return target === this.current || this.current.contains(target); }
  value() { return this.alive ? this.current.value : ""; }
  setValue(value) { if (this.alive) this.current.value=value; }
  selectionStart() { return this.current.selectionStart; }
  selectionEnd() { return this.current.selectionEnd; }
  setSelectionRange(start,end) { this.current.setSelectionRange(start,end); }
  setReadOnly(value) {this.readOnly=value;this.current.readOnly=value;}
  setSpellcheck(value) {this.spellcheck=value;this.current.spellcheck=this.secure?false:value;}
  setAttribute(name,value) {
    this.attributes.set(name,value);
    if (this.secure && ["autocorrect","autocapitalize","autocomplete"].includes(name)) value="off";
    this.current.setAttribute(name,value);
  }
  removeAttribute(name) {this.attributes.delete(name);this.current.removeAttribute(name);}
  getAttribute(name) {return this.current.getAttribute(name);}
  focus() { if (this.alive && this.ownerActive && !this.isFocused()) this.current.focus({preventScroll:true}); }
  blur() { if (this.alive) this.current.blur(); }
  isFocused() {return this.document.activeElement === this.current;}
  remove() {
    if (!this.alive) return;
    this.alive=false;
    this.current.value="";
    this.host.remove();
    this.attributes.clear();
  }
}

export class GpuiSemanticDom {
  constructor(document, body, canvas, ime, dispatch) {
    this.document=document;this.canvas=canvas;this.ime=ime;this.dispatch=dispatch;
    this.nodes=new Map();this.elements=new Map();this.listeners=[];
    this.alive=true;this.restoring=false;this.pending=null;this.composing=false;
    this.root=document.createElement("div");
    this.root.setAttribute("data-gpui-semantic-root","");
    Object.assign(this.root.style,{position:"fixed",top:"0",left:"0",width:"0",height:"0",opacity:"0",pointerEvents:"none"});
    body.appendChild(this.root);
    this.canvasHidden=canvas.getAttribute("aria-hidden");
    this.canvasTab=canvas.getAttribute("tabindex");
    canvas.setAttribute("aria-hidden","true");canvas.setAttribute("tabindex","-1");
    this.listen("focusin",event=>{
      if (this.restoring) return;
      const node=this.eventNode(event);if(node) {
        this.action(node,"ScrollIntoView");this.action(node,"Focus");
      }
    });
    this.listen("click",event=>{
      const node=this.eventNode(event);if(node && this.action(node,"Click")) event.preventDefault();
    });
    this.listen("contextmenu",event=>{
      const node=this.eventNode(event);if(node && this.action(node,"ShowContextMenu")) event.preventDefault();
    });
    this.listen("input",event=>{
      const node=this.eventNode(event);if(!node || node.kind!=="range") return;
      const value=Number(event.target.value);
      if(Number.isFinite(value)) {
        if(!this.action(node,"SetValue",value)) this.action(node,value>Number(node.numeric)?"Increment":"Decrement");
      }
      event.target.value=String(node.numeric);
    });
    this.listen("keydown",event=>{
      const node=this.eventNode(event);
      if(!node || event.isComposing || event.keyCode===229 || event.altKey || event.ctrlKey || event.metaKey) return;
      if(node.kind==="range" || node.kind==="spin") {
        const command=["ArrowRight","ArrowUp","PageUp"].includes(event.key)?"Increment":["ArrowLeft","ArrowDown","PageDown"].includes(event.key)?"Decrement":null;
        if(command && this.action(node,command)) event.preventDefault();
        if(event.key==="Home" && this.action(node,"SetValue",node.min)) event.preventDefault();
        if(event.key==="End" && this.action(node,"SetValue",node.max)) event.preventDefault();
      }
      if(event.key==="ArrowRight" && node.attrs["aria-expanded"]==="false" && this.action(node,"Expand")) event.preventDefault();
      if(event.key==="ArrowLeft" && node.attrs["aria-expanded"]==="true" && this.action(node,"Collapse")) event.preventDefault();
    });
  }
  listen(name,callback) {this.root.addEventListener(name,callback);this.listeners.push([name,callback]);}
  cssBounds(bounds) {
  const [x,y,width,height]=bounds||[0,0,1,1];
  const rect=this.canvas.getBoundingClientRect?.()||{left:0,top:0,width:0,height:0};
  const scaleX=rect.width>0 && Number(this.canvas.width)>0?Number(this.canvas.width)/rect.width:1;
  const scaleY=rect.height>0 && Number(this.canvas.height)>0?Number(this.canvas.height)/rect.height:1;
  return [Number(rect.left||0)+x/scaleX,Number(rect.top||0)+y/scaleY,width/scaleX,height/scaleY];
 }
 id(id) {return "gpui-ax-"+id;}
  eventNode(event) {
    if(!this.alive) return null;
    let element=event.target;
    while(element && element!==this.root) {
      const id=element.getAttribute && element.getAttribute("data-gpui-node");
      if(id && this.elements.get(id)===element) return this.nodes.get(id)||null;
      element=element.parentNode;
    }
    return null;
  }
  action(node,action,data=null) {
    if(!this.alive || this.composing || node.disabled || !node.actions.includes(action) || this.nodes.get(node.id)!==node) return false;
    if(node.readOnly && ["SetValue","ReplaceSelectedText"].includes(action)) return false;
    this.dispatch(node.id,action,data);return true;
  }
  ownsTarget(target) {return this.alive && this.root.contains(target);}
  nativeKeyboard(event) {
    const node=this.eventNode(event);if(!node || node.disabled) return false;
    if(event.altKey || event.ctrlKey || event.metaKey || event.isComposing || event.keyCode===229) return false;
    // Combobox Enter belongs to GPUI: native button activation bypasses its key bindings.
    if(node.attrs.role==="combobox" && event.key==="Enter") return false;
    return ((node.kind==="button" || node.kind==="checkbox" || node.kind==="radio") && ["Enter"," "].includes(event.key)) ||
      (node.kind==="range" && ["ArrowLeft","ArrowRight","ArrowUp","ArrowDown","Home","End","PageUp","PageDown"].includes(event.key));
  }
  update(snapshot,composing) {
    if(!this.alive) return;
    this.pending=snapshot;this.composing=composing;
    // Actions always consult the newest authoritative snapshot, even while DOM
    // mutations are deferred to avoid perturbing an active input composition.
    this.nodes=new Map(snapshot.nodes.map(node=>[node.id,node]));
    // GPUI commits this after the current native input owner/security mode is
    // installed, never from the middle of painting/accessibility collection.
  }
  frameReady() {if(!this.composing && this.pending) this.reconcile();}
  compositionStarted() {this.composing=true;}
  ownerChanged() {this.composing=false;}
  compositionEnded() {this.composing=false;if(this.pending) this.reconcile();}
  failClosed() {this.nodes.clear();this.root.setAttribute("aria-hidden","true");}
  reconcile() {
    const snapshot=this.pending;if(!snapshot) return;
    this.root.removeAttribute("aria-hidden");
    for(const [id,element] of this.elements) if(!this.nodes.has(id)) {element.remove();this.elements.delete(id);}
    for(const node of snapshot.nodes) {
      let element=this.elements.get(node.id);
      const tag=node.kind==="button"?"button":["range","checkbox","radio","protected"].includes(node.kind)?"input":"div";
      if(element && element.tagName.toLowerCase()!==tag) {element.remove();this.elements.delete(node.id);element=null;}
      if(!element) {element=this.document.createElement(tag);element.setAttribute("id",this.id(node.id));element.setAttribute("data-gpui-node",node.id);this.elements.set(node.id,element);}
      if(tag==="button") element.setAttribute("type","button");
      if(tag==="input") element.setAttribute("type",node.kind==="protected"?"password":node.kind);
      if(node.kind==="protected") {element.readOnly=true;element.value="";}
      // Remove stale attributes as names/states/relations change. Application
      // strings are exclusively textContent and attribute values, never HTML.
      const retained=new Set(["id","data-gpui-node","type","tabindex","aria-hidden","aria-owns"]);
      for(const attr of Array.from(element.attributes)) if(!retained.has(attr.name)&&!(attr.name in node.attrs)) element.removeAttribute(attr.name);
      for(const [name,value] of Object.entries(node.attrs)) {if(node.kind==="protected" && name.startsWith("aria-value")) {element.removeAttribute(name);continue;}attribute(element,name,value);}
      attribute(element,"tabindex",node.focusable && !node.disabled?"0":"-1");
      if(tag==="button" || tag==="input") element.disabled=node.disabled;
      element.onclick=event=>{const current=this.eventNode(event);if(current&&this.action(current,"Click")){event.preventDefault();event.stopPropagation();}};
      if(node.kind==="range") {
        element.min=String(node.min);element.max=String(node.max);element.step=String(node.step);element.value=String(node.numeric);
      }
      if(node.kind==="checkbox" || node.kind==="radio") {element.checked=node.checked==="true";element.indeterminate=node.checked==="mixed";}
      // Only a dedicated text span owns projected text. Never replace an entire
      // retained subtree's textContent while its descendants have focus.
      if(tag!=="input") {
        if(!element._gpuiText) {element._gpuiText=this.document.createElement("span");element.appendChild(element._gpuiText);}
        if(element._gpuiText.textContent!==(node.text||"")) element._gpuiText.textContent=node.text||"";
      }
      const [x,y,width,height]=this.cssBounds(node.bounds);
      Object.assign(element.style,{position:"fixed",left:x+"px",top:y+"px",width:width+"px",height:height+"px",pointerEvents:"none"});
    }
    const byId=this.elements;
    for(const node of snapshot.nodes) {
      const element=byId.get(node.id);
      // AccessKit children on native input leaves are text-run internals; native
      // controls/IME expose their own values/selection instead of duplicate runs.
      if(["range","checkbox","radio","text","protected"].includes(node.kind)) continue;
      const wanted=node.children.filter(id=>byId.has(id));
      let cursor=element._gpuiText.nextSibling;
      for(const id of wanted) {
        const child=byId.get(id);
        if(child!==cursor) element.insertBefore(child,cursor);else cursor=cursor.nextSibling;
      }
    }
    const root=byId.get(snapshot.root);
    if(root && root.parentNode!==this.root) this.root.appendChild(root);
    const focused=this.nodes.get(snapshot.focus);
    // The input-handler metadata normally selects this before focus/sync.
    // Fail safe for a custom producer advertising a protected AccessKit role.
    if(focused?.kind==="protected" && !this.ime.isSecure()) this.ime.setSecure(true);
    const mirror=this.ime.activeElement();
    mirror.setAttribute("id","gpui-ime-owner");
    const textFocused=this.ime.hasOwner() && focused && ["text","protected"].includes(focused.kind);
    for(const node of snapshot.nodes) {
      const element=byId.get(node.id);
      if(["text","protected"].includes(node.kind)) attribute(element,"aria-hidden",textFocused && node.id===focused.id?"true":"false");
      const children=node.children.map(id=>textFocused&&id===focused.id?"gpui-ime-owner":this.id(id));
      if(children.length) attribute(element,"aria-owns",children.join(" "));else element.removeAttribute("aria-owns");
    }
    if(textFocused) {
      mirror.removeAttribute("aria-hidden");
      this.ime.setSemanticBounds(this.cssBounds(focused.bounds));
      if(focused.kind==="protected") {
        mirror.removeAttribute("role");for(const name of ["aria-valuetext","aria-valuenow","value"]) mirror.removeAttribute(name);
      } else if(focused.attrs.role) attribute(mirror,"role",focused.attrs.role);
      for(const name of ["aria-label","aria-description","aria-required","aria-invalid","aria-readonly","aria-multiline","aria-expanded","aria-controls","aria-activedescendant"]) {
        if(focused.attrs[name]!==undefined) attribute(mirror,name,focused.attrs[name]);else mirror.removeAttribute(name);
      }
    }
    this.restoring=true;
    try {
      if(textFocused) this.ime.focus();
      else {
        const target=byId.get(snapshot.focus)||root;
        if(target && this.document.activeElement!==target) target.focus({preventScroll:true});
        mirror.setAttribute("aria-hidden","true");
      }
    } finally {this.restoring=false;}
  }
  close() {
    if(!this.alive) return;
    this.alive=false;
    for(const [name,callback] of this.listeners) this.root.removeEventListener(name,callback);
    this.listeners=[];this.nodes.clear();this.elements.clear();this.pending=null;this.dispatch=null;
    this.root.remove();
    for(const [name,value] of [["aria-hidden",this.canvasHidden],["tabindex",this.canvasTab]]) {
      if(value===null) this.canvas.removeAttribute(name);else this.canvas.setAttribute(name,value);
    }
  }
}

export function createImeHost(document,body,touch) {return new GpuiImeHost(document,body,touch);}
export function createSemanticDom(document,body,canvas,ime,dispatch) {return new GpuiSemanticDom(document,body,canvas,ime,dispatch);}
