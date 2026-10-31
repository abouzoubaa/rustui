//! Browser-owned native semantic controls and the one GPUI-owned IME host.
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/src/a11y_dom.js")]
extern "C" {
    #[wasm_bindgen(js_name = GpuiImeHost)]
    #[derive(Clone)]
    pub(crate) type ImeHost;
    #[wasm_bindgen(catch, js_name=createImeHost)]
    pub(crate) fn create_ime_host(
        document: &web_sys::Document,
        body: &web_sys::HtmlElement,
        touch: bool,
    ) -> Result<ImeHost, JsValue>;
    #[wasm_bindgen(method, js_name = setOwnerActive)]
    pub(crate) fn set_owner_active(this: &ImeHost, active: bool);
    #[wasm_bindgen(method, js_name = ownerChanged)]
    pub(crate) fn owner_changed(this: &ImeHost);
    #[wasm_bindgen(method, js_name=setSecure)]
    pub(crate) fn set_secure(this: &ImeHost, secure: bool);
    #[wasm_bindgen(method, js_name=isSecure)]
    pub(crate) fn is_secure(this: &ImeHost) -> bool;
    #[wasm_bindgen(method, js_name=activeElement)]
    pub(crate) fn active_element(this: &ImeHost) -> web_sys::HtmlElement;
    #[wasm_bindgen(method, js_name=eventTarget)]
    pub(crate) fn event_target(this: &ImeHost) -> web_sys::EventTarget;
    #[wasm_bindgen(method, js_name=containsTarget)]
    pub(crate) fn contains_target(this: &ImeHost, target: &web_sys::EventTarget) -> bool;
    #[wasm_bindgen(method)]
    pub(crate) fn value(this: &ImeHost) -> String;
    #[wasm_bindgen(method, js_name=setValue)]
    pub(crate) fn set_value(this: &ImeHost, value: &str);
    #[wasm_bindgen(method, catch, js_name=selectionStart)]
    pub(crate) fn selection_start(this: &ImeHost) -> Result<Option<u32>, JsValue>;
    #[wasm_bindgen(method, catch, js_name=selectionEnd)]
    pub(crate) fn selection_end(this: &ImeHost) -> Result<Option<u32>, JsValue>;
    #[wasm_bindgen(method, catch, js_name=setSelectionRange)]
    pub(crate) fn set_selection_range(this: &ImeHost, start: u32, end: u32) -> Result<(), JsValue>;
    #[wasm_bindgen(method, js_name=setReadOnly)]
    pub(crate) fn set_read_only(this: &ImeHost, value: bool);
    #[wasm_bindgen(method, js_name=setSpellcheck)]
    pub(crate) fn set_spellcheck(this: &ImeHost, value: bool);
    #[wasm_bindgen(method, catch, js_name=setAttribute)]
    pub(crate) fn set_attribute(this: &ImeHost, name: &str, value: &str) -> Result<(), JsValue>;
    #[wasm_bindgen(method, catch, js_name=removeAttribute)]
    pub(crate) fn remove_attribute(this: &ImeHost, name: &str) -> Result<(), JsValue>;
    #[wasm_bindgen(method, js_name=getAttribute)]
    pub(crate) fn get_attribute(this: &ImeHost, name: &str) -> Option<String>;
    #[wasm_bindgen(method)]
    pub(crate) fn focus(this: &ImeHost);
    #[wasm_bindgen(method, catch)]
    pub(crate) fn blur(this: &ImeHost) -> Result<(), JsValue>;
    #[wasm_bindgen(method, js_name=isFocused)]
    pub(crate) fn is_focused(this: &ImeHost) -> bool;
    #[wasm_bindgen(method)]
    pub(crate) fn remove(this: &ImeHost);

    #[wasm_bindgen(js_name = GpuiSemanticDom)]
    pub(crate) type SemanticDom;
    #[wasm_bindgen(catch, js_name=createSemanticDom)]
    pub(crate) fn create_semantic_dom(
        document: &web_sys::Document,
        body: &web_sys::HtmlElement,
        canvas: &web_sys::HtmlCanvasElement,
        ime: &ImeHost,
        dispatch: &js_sys::Function,
    ) -> Result<SemanticDom, JsValue>;
    #[wasm_bindgen(method, catch)]
    pub(crate) fn update(
        this: &SemanticDom,
        snapshot: &JsValue,
        composing: bool,
    ) -> Result<(), JsValue>;
    #[wasm_bindgen(method, js_name=ownsTarget)]
    pub(crate) fn owns_target(this: &SemanticDom, target: &web_sys::EventTarget) -> bool;
    #[wasm_bindgen(method, js_name=nativeKeyboard)]
    pub(crate) fn native_keyboard(this: &SemanticDom, event: &web_sys::KeyboardEvent) -> bool;
    #[wasm_bindgen(method, catch, js_name=frameReady)]
    pub(crate) fn frame_ready(this: &SemanticDom) -> Result<(), JsValue>;
    #[wasm_bindgen(method, js_name=compositionStarted)]
    pub(crate) fn composition_started(this: &SemanticDom);
    #[wasm_bindgen(method, js_name=ownerChanged)]
    pub(crate) fn semantic_owner_changed(this: &SemanticDom);
    #[wasm_bindgen(method, catch, js_name=compositionEnded)]
    pub(crate) fn composition_ended(this: &SemanticDom) -> Result<(), JsValue>;
    #[wasm_bindgen(method, js_name=failClosed)]
    pub(crate) fn fail_closed(this: &SemanticDom);
    #[wasm_bindgen(method)]
    pub(crate) fn close(this: &SemanticDom);
}
