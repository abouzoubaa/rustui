import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
const code=readFileSync(new URL('../../vendor/gpui-pre-web-0.3.9/src/a11y_dom.js',import.meta.url),'utf8');
const {GpuiImeHost,GpuiSemanticDom}=await import('data:text/javascript;base64,'+Buffer.from(code).toString('base64'));

// Deterministic DOM protocol fixture, not a browser/AX implementation. These
// tests execute the actual candidate JS and do not infer screen-reader support.
class Element {
  constructor(document,tag) {this.document=document;this.tagName=tag.toUpperCase();this.parentNode=null;this.children=[];this.attrs=new Map();this.style={};this.listeners=new Map();this._value='';this.selectionStart=0;this.selectionEnd=0;this._text='';}
  get attributes(){return [...this.attrs].map(([name,value])=>({name,value}));}
  get nextSibling(){if(!this.parentNode)return null;return this.parentNode.children[this.parentNode.children.indexOf(this)+1]||null;}
  setAttribute(name,value){this.attrs.set(name,String(value));}
  getAttribute(name){return this.attrs.get(name)??null;}
  removeAttribute(name){this.attrs.delete(name);}
  appendChild(child){return this.insertBefore(child,null);}
  insertBefore(child,next){if(child===next)return child;child.remove();child.parentNode=this;const at=next===null?this.children.length:this.children.indexOf(next);assert(at>=0);this.children.splice(at,0,child);return child;}
  remove(){if(this.parentNode){const at=this.parentNode.children.indexOf(this);this.parentNode.children.splice(at,1);this.parentNode=null;}}
  contains(target){return target===this||this.children.some(child=>child.contains(target));}
  addEventListener(name,callback){if(!this.listeners.has(name))this.listeners.set(name,new Set());this.listeners.get(name).add(callback);}
  removeEventListener(name,callback){this.listeners.get(name)?.delete(callback);}
  dispatch(name,properties={}){const event={target:this,key:'',keyCode:0,isComposing:false,altKey:false,ctrlKey:false,metaKey:false,defaultPrevented:false,stopped:false,preventDefault(){this.defaultPrevented=true;},stopPropagation(){this.stopped=true;},...properties};for(let at=this;at;at=at.parentNode){if(name==='click'&&at.onclick)at.onclick(event);for(const callback of at.listeners.get(name)||[])callback(event);if(event.stopped)break;}return event;}
  focus(){this.document.activeElement=this;this.dispatch('focusin');}
  blur(){if(this.document.activeElement===this)this.document.activeElement=null;}
  get value(){return this._value;}
  set value(value){this._value=this.tagName==='INPUT'?String(value).replace(/[\r\n]/g,''):String(value);}
  setSelectionRange(start,end){this.selectionStart=start;this.selectionEnd=end;}
  get textContent(){return this._text+this.children.map(child=>child.textContent).join('');}
  set textContent(value){this.children=[];this._text=String(value);}
  set innerHTML(_){throw new Error('application text must never enter an HTML parser');}
}
class FrameAdapter extends GpuiSemanticDom {update(snapshot,composing){super.update(snapshot,composing);super.frameReady();}}
class Document {constructor(){this.activeElement=null;this.body=new Element(this,'body');}createElement(tag){return new Element(this,tag);}}
function fixture(){const document=new Document();const canvas=document.createElement('canvas');canvas.setAttribute('tabindex','0');document.body.appendChild(canvas);const ime=new GpuiImeHost(document,document.body,false);ime.setOwnerActive(true);const requests=[];const dom=new FrameAdapter(document,document.body,canvas,ime,(...request)=>requests.push(request));return {document,canvas,ime,dom,requests};}
function node(id,extra={}){return {id,kind:'generic',attrs:{role:'group'},actions:[],children:[],disabled:false,readOnly:false,focusable:false,text:'',bounds:[10,20,100,30],...extra};}
function snapshot(nodes,focus='root'){return {root:'root',focus,nodes:[node('root',{attrs:{role:'application','aria-label':'Test application'},children:nodes.map(node=>node.id)}),...nodes]};}


test('one active textarea preserves multiline value and UTF-16 selection',()=>{const {ime}=fixture();ime.setValue('alpha\nbeta');ime.setSelectionRange(2,7);assert.equal(ime.activeElement().tagName,'TEXTAREA');assert.equal(ime.value(),'alpha\nbeta');assert.deepEqual([ime.selectionStart(),ime.selectionEnd()],[2,7]);assert.equal(ime.host.children.length,1);});
test('secure transition creates a native password owner and clears detached plaintext',()=>{const {ime}=fixture();ime.setValue('public fixture');const old=ime.activeElement();ime.setSecure(true);assert.equal(old.value,'');assert.equal(old.parentNode,null);assert.equal(ime.activeElement().tagName,'INPUT');assert.equal(ime.activeElement().getAttribute('type'),'password');assert.equal(ime.value(),'');assert.equal(ime.host.children.length,1);ime.setSpellcheck(true);ime.setAttribute('autocorrect','on');assert.equal(ime.activeElement().spellcheck,false);assert.equal(ime.getAttribute('autocorrect'),'off');});
test('owner replacement never copies the previous value or accepts its stale element',()=>{const {ime}=fixture();ime.setSecure(true);ime.setValue('protected fixture sentinel');const old=ime.activeElement();const host=ime.eventTarget();ime.ownerChanged();assert.equal(ime.eventTarget(),host);assert.equal(old.value,'');assert.equal(ime.value(),'');assert.equal(ime.containsTarget(old),false);assert.equal(ime.containsTarget(ime.activeElement()),true);assert.equal(ime.host.children.length,1);});
test('names and values remain plain text and stale attributes are removed',()=>{const {dom}=fixture();const button=node('b',{kind:'button',attrs:{role:'button','aria-label':'<img onerror=fixture>','aria-expanded':'true'},text:'<script>fixture</script>',actions:['Click','Focus'],focusable:true});dom.update(snapshot([button]),false);const element=dom.elements.get('b');assert.equal(element.getAttribute('aria-label'),'<img onerror=fixture>');assert.equal(element.textContent,'<script>fixture</script>');dom.update(snapshot([{...button,attrs:{role:'button','aria-label':'Updated name'},text:'Updated visible text'}]),false);assert.equal(dom.elements.get('b'),element);assert.equal(element.getAttribute('aria-label'),'Updated name');assert.equal(element.getAttribute('aria-expanded'),null);assert.equal(element.textContent,'Updated visible text');});
test('retained focused children are not rebuilt on an ancestor name update',()=>{const {dom,document,requests}=fixture();const child=node('b',{kind:'button',attrs:{role:'button','aria-label':'Child'},actions:['Click','Focus'],focusable:true});dom.update(snapshot([child],'b'),false);const element=dom.elements.get('b');assert.equal(document.activeElement,element);assert.deepEqual(requests,[]);const next=snapshot([child],'b');next.nodes[0].attrs['aria-label']='Renamed root';dom.update(next,false);assert.equal(dom.elements.get('b'),element);assert.equal(document.activeElement,element);assert.deepEqual(requests,[]);});
test('click and focus route one supported callback to the current authoritative ID',()=>{const {dom,requests}=fixture();dom.update(snapshot([node('b',{kind:'button',attrs:{role:'button','aria-label':'Activate'},actions:['Click','Focus','ScrollIntoView'],focusable:true})]),false);const element=dom.elements.get('b');element.focus();assert.deepEqual(requests.map(r=>r.slice(0,2)),[['b','ScrollIntoView'],['b','Focus']]);requests.length=0;element.dispatch('click');assert.deepEqual(requests,[['b','Click',null]]);});
test('disabled and unsupported actions do not dispatch',()=>{const {dom,requests}=fixture();dom.update(snapshot([node('disabled',{kind:'button',attrs:{role:'button','aria-disabled':'true'},actions:['Click','Focus'],focusable:true,disabled:true}),node('plain')]),false);dom.elements.get('disabled').dispatch('click');dom.elements.get('disabled').focus();dom.elements.get('plain').dispatch('click');assert.deepEqual(requests,[]);assert.equal(dom.elements.get('disabled').getAttribute('tabindex'),'-1');});
test('removed IDs and queued old click handlers cannot act or accumulate',()=>{const {dom,requests}=fixture();let old;for(let index=0;index<100;index++){const id='node-'+index;dom.update(snapshot([node(id,{kind:'button',attrs:{role:'button'},actions:['Click']})]),false);if(old)old.dispatch('click');old=dom.elements.get(id);assert.equal(dom.elements.size,2);assert.equal(dom.nodes.size,2);}assert.deepEqual(requests,[]);});
test('native range input uses SetValue and keyboard increment has one route',()=>{const {dom,requests}=fixture();dom.update(snapshot([node('s',{kind:'range',attrs:{role:'slider','aria-label':'Volume'},actions:['Focus','SetValue','Increment','Decrement'],focusable:true,numeric:.5,min:0,max:1,step:.125})]),false);const element=dom.elements.get('s');element.value='.75';element.dispatch('input');assert.deepEqual(requests,[['s','SetValue',.75]]);assert.equal(element.value,'0.5');requests.length=0;const event=element.dispatch('keydown',{key:'ArrowUp'});assert.equal(event.defaultPrevented,true);assert.deepEqual(requests,[['s','Increment',null]]);assert.equal(dom.nativeKeyboard({...event,key:'ArrowUp'}),true);});
test('composition defers DOM changes while stale actions are rejected',()=>{const {dom,ime,requests}=fixture();ime.setValue('composition fixture');ime.setSelectionRange(2,4);dom.update(snapshot([node('old',{kind:'button',attrs:{role:'button','aria-label':'Old'},actions:['Click']})]),false);const old=dom.elements.get('old');dom.update(snapshot([node('new',{kind:'button',attrs:{role:'button','aria-label':'New'},actions:['Click']})]),true);old.dispatch('click');assert.deepEqual(requests,[]);assert.equal(dom.elements.has('old'),true);dom.compositionEnded();assert.equal(dom.elements.has('old'),false);assert.equal(dom.elements.has('new'),true);assert.equal(ime.value(),'composition fixture');assert.deepEqual([ime.selectionStart(),ime.selectionEnd()],[2,4]);});
test('protected semantic proxies never copy a protected value or value attributes',()=>{const {dom,ime}=fixture();ime.setSecure(true);ime.setValue('protected fixture sentinel');dom.update(snapshot([node('secret',{kind:'protected',attrs:{role:'textbox','aria-label':'Password','aria-valuetext':'must not be projected'},text:'must not be projected',actions:['Focus'],focusable:true})],'secret'),false);const proxy=dom.elements.get('secret');assert.equal(proxy.tagName,'INPUT');assert.equal(proxy.getAttribute('type'),'password');assert.equal(proxy.readOnly,true);assert.equal(proxy.value,'');assert.equal(proxy.textContent,'');assert.equal(proxy.getAttribute('aria-valuetext'),null);assert.equal(ime.value(),'protected fixture sentinel');assert.equal(ime.activeElement().getAttribute('type'),'password');assert.equal(proxy.getAttribute('aria-hidden'),'true');assert.equal(ime.activeElement().getAttribute('aria-hidden'),null);assert.equal(dom.elements.get('root').getAttribute('aria-owns'),'gpui-ime-owner');});
test('close removes semantic listeners/nodes and queued callbacks become inert',()=>{const {dom,canvas,requests,ime}=fixture();dom.update(snapshot([node('b',{kind:'button',attrs:{role:'button'},actions:['Click']})]),false);const old=dom.elements.get('b');dom.close();dom.close();old.dispatch('click');assert.deepEqual(requests,[]);assert.equal(dom.root.parentNode,null);assert.equal(dom.elements.size,0);assert.equal(dom.nodes.size,0);assert.equal(canvas.getAttribute('tabindex'),'0');assert.equal(canvas.getAttribute('aria-hidden'),null);ime.remove();assert.equal(ime.host.parentNode,null);assert.equal(ime.value(),'');});

test('semantic focus waits for the secure native owner frame boundary',()=>{const document=new Document();const canvas=document.createElement('canvas');document.body.appendChild(canvas);const ime=new GpuiImeHost(document,document.body,false);ime.setOwnerActive(true);const dom=new GpuiSemanticDom(document,document.body,canvas,ime,()=>{});dom.update(snapshot([node('secret',{kind:'protected',attrs:{role:'textbox','aria-label':'Password'},actions:['Focus'],focusable:true})],'secret'),false);assert.equal(dom.elements.size,0);assert.equal(ime.activeElement().tagName,'TEXTAREA');ime.ownerChanged();ime.setSecure(true);ime.setOwnerActive(true);dom.frameReady();assert.equal(ime.activeElement().getAttribute('type'),'password');assert.equal(document.activeElement,ime.activeElement());assert.equal(ime.value(),'');});
test('composition start blocks native semantic actions without waiting for a redraw',()=>{const {dom,requests}=fixture();dom.update(snapshot([node('b',{kind:'button',attrs:{role:'button'},actions:['Click']})]),false);dom.compositionStarted();dom.elements.get('b').dispatch('click');assert.deepEqual(requests,[]);dom.ownerChanged();dom.frameReady();dom.elements.get('b').dispatch('click');assert.deepEqual(requests,[['b','Click',null]]);});

test('idle native owner starts hidden disabled and cannot steal focus',()=>{
 const document=new Document();const ime=new GpuiImeHost(document,document.body,false);
 assert.equal(document.activeElement,null);assert.equal(ime.hasOwner(),false);
 assert.equal(ime.activeElement().getAttribute('aria-hidden'),'true');
 assert.equal(ime.activeElement().getAttribute('tabindex'),'-1');assert.equal(ime.activeElement().disabled,true);
 ime.focus();assert.equal(document.activeElement,null);
});
test('native owner exposure requires installed current text owner',()=>{
 const {dom,ime,document}=fixture();ime.setOwnerActive(false);
 dom.update(snapshot([node('text',{kind:'text',attrs:{role:'textbox','aria-label':'Search'},actions:['Focus'],focusable:true})],'text'),false);
 assert.equal(ime.activeElement().getAttribute('aria-hidden'),'true');assert.notEqual(document.activeElement,ime.activeElement());
 ime.setOwnerActive(true);dom.frameReady();
 assert.equal(ime.activeElement().getAttribute('aria-hidden'),null);assert.equal(document.activeElement,ime.activeElement());
 ime.setOwnerActive(false);assert.equal(ime.activeElement().getAttribute('aria-hidden'),'true');assert.notEqual(document.activeElement,ime.activeElement());
 ime.focus();assert.notEqual(document.activeElement,ime.activeElement());
 ime.ownerChanged();assert.equal(ime.hasOwner(),false);assert.equal(ime.activeElement().getAttribute('aria-hidden'),'true');assert.equal(ime.activeElement().disabled,true);
});
test('physical AccessKit bounds normalize to CSS viewport and native owner',()=>{
 const {dom,ime,canvas}=fixture();canvas.width=2560;canvas.height=1440;
 canvas.getBoundingClientRect=()=>({left:17,top:29,width:1280,height:720});
 dom.update(snapshot([node('text',{kind:'text',attrs:{role:'textbox','aria-label':'Search'},bounds:[26,253,446,48],actions:['Focus'],focusable:true})],'text'),false);
 assert.equal(dom.elements.get('text').style.left,'30px');assert.equal(dom.elements.get('text').style.top,'155.5px');
 assert.equal(dom.elements.get('text').style.width,'223px');assert.equal(dom.elements.get('text').style.height,'24px');
 assert.equal(ime.host.style.left,'30px');assert.equal(ime.host.style.top,'155.5px');assert.equal(ime.host.style.width,'223px');assert.equal(ime.host.style.height,'24px');
});

test('native replacement input retains one owner and the replacement selection',()=>{
 const {dom,ime,document}=fixture();
 dom.update(snapshot([node('text',{kind:'text',attrs:{role:'textbox','aria-label':'Search'},actions:['Focus'],focusable:true})],'text'),false);
 const owner=ime.activeElement();const observed=[];
 ime.eventTarget().addEventListener('beforeinput',e=>{if(ime.containsTarget(e.target))observed.push(['beforeinput',ime.selectionStart(),ime.selectionEnd()]);});
 ime.eventTarget().addEventListener('input',e=>{if(ime.containsTarget(e.target))observed.push(['input',ime.value(),ime.selectionStart(),ime.selectionEnd()]);});
 owner.value='tree';owner.setSelectionRange(0,4);owner.dispatch('beforeinput',{inputType:'insertText',data:'button'});
 owner.value='button';owner.setSelectionRange(6,6);owner.dispatch('input',{inputType:'insertText',data:'button'});
 dom.frameReady();
 assert.deepEqual(observed,[['beforeinput',0,4],['input','button',6,6]]);
 assert.equal(ime.activeElement(),owner);assert.equal(document.activeElement,owner);assert.equal(ime.value(),'button');assert.equal(ime.selectionStart(),6);
});
test('repeated native replacement and empty input never rebuild or append the owner',()=>{
 const {dom,ime}=fixture();dom.update(snapshot([node('text',{kind:'text',attrs:{role:'textbox','aria-label':'Search'},actions:['Focus'],focusable:true})],'text'),false);
 const owner=ime.activeElement();owner.value='tree';
 for(const replacement of ['button','button','']) {
  owner.setSelectionRange(0,owner.value.length);owner.dispatch('beforeinput',{inputType:'insertText',data:replacement});
  owner.value=replacement;owner.setSelectionRange(replacement.length,replacement.length);owner.dispatch('input',{inputType:'insertText',data:replacement});
  dom.frameReady();assert.equal(ime.activeElement(),owner);assert.equal(ime.value(),replacement);assert.equal(ime.selectionStart(),replacement.length);
 }
});
