//! Browser lifetime/origin boundary; model work runs through GPUI's executor.
use crate::preview_transport::{self, Delivery, Reply};
use rustui::{App, view::OwnerAction};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::{JsValue, closure::Closure, prelude::wasm_bindgen};

#[wasm_bindgen(module = "/web/preview-transport.js")]
extern "C" {
    #[wasm_bindgen(js_name=requestedProfile)]
    pub(crate) fn requested_profile() -> String;
    #[wasm_bindgen(catch,js_name=freshToken)]
    fn fresh_token() -> Result<String, JsValue>;
    #[wasm_bindgen(catch,js_name=createBridge)]
    fn create_bridge(receive: &JsValue, disconnected: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(js_name=markReady)]
    fn mark_ready(handle: &JsValue);
    #[wasm_bindgen(js_name=postReply)]
    fn post_reply(handle: &JsValue, wire: &str, session: &str);
    #[wasm_bindgen(js_name=closeBridge)]
    fn close_bridge(handle: &JsValue);
}

pub(crate) struct Bridge<C: 'static> {
    inner: Rc<Inner<C>>,
}
struct Inner<C: 'static> {
    owner: RefCell<Option<OwnerAction<C>>>,
    handler: fn(&mut C, Delivery) -> (Reply, bool),
    disconnected: fn(&mut C) -> bool,
    listener: RefCell<Option<Listener>>,
    closed: Cell<bool>,
    busy: Cell<bool>,
}
struct Listener {
    handle: JsValue,
    _receive: Closure<dyn FnMut(String)>,
    _disconnected: Closure<dyn FnMut()>,
}
impl Drop for Listener {
    fn drop(&mut self) {
        close_bridge(&self.handle);
    }
}
impl<C: 'static> Clone for Bridge<C> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
impl<C: 'static> Bridge<C> {
    pub fn new(
        handler: fn(&mut C, Delivery) -> (Reply, bool),
        disconnected: fn(&mut C) -> bool,
    ) -> Self {
        Self {
            inner: Rc::new(Inner {
                owner: RefCell::new(None),
                handler,
                disconnected,
                listener: RefCell::new(None),
                closed: Cell::new(false),
                busy: Cell::new(false),
            }),
        }
    }
    pub fn install(&self, cx: &mut App) {
        if self.inner.closed.get() || self.inner.listener.borrow().is_some() {
            return;
        }
        let weak = Rc::downgrade(&self.inner);
        let app = cx.to_async();
        let executor = cx.foreground_executor().clone();
        let receive: Closure<dyn FnMut(String)> = Closure::new(move |wire: String| {
            let Some(inner) = weak.upgrade().filter(|inner| !inner.closed.get()) else {
                return;
            };
            let request = match preview_transport::decode(&wire) {
                Ok(request) => request,
                Err(code) => {
                    inner.send(Reply::malformed(code));
                    return;
                }
            };
            let unavailable = Reply::unavailable(&request);
            if inner.busy.replace(true) {
                inner.send(Reply::failure(&request, "busy"));
                return;
            }
            let Some(owner) = inner.owner.borrow().clone() else {
                inner.busy.set(false);
                inner.send(unavailable);
                return;
            };
            let fresh = if request.is_connect() {
                match fresh_token() {
                    Ok(token) => token,
                    Err(_) => {
                        inner.busy.set(false);
                        inner.send(Reply::failure(&request, "session_unavailable"));
                        return;
                    }
                }
            } else {
                String::new()
            };
            let handler = inner.handler;
            let pending = Rc::downgrade(&inner);
            let app = app.clone();
            executor
                .spawn(async move {
                    let Some(inner) = pending.upgrade().filter(|inner| !inner.closed.get()) else {
                        return;
                    };
                    let reply = app
                        .update(|cx| owner.dispatch_with(Delivery { request, fresh }, handler, cx))
                        .unwrap_or(unavailable);
                    inner.send(reply);
                    inner.busy.set(false);
                })
                .detach();
        });
        let weak = Rc::downgrade(&self.inner);
        let app = cx.to_async();
        let executor = cx.foreground_executor().clone();
        let disconnected: Closure<dyn FnMut()> = Closure::new(move || {
            let Some(inner) = weak.upgrade() else {
                return;
            };
            if inner.closed.replace(true) {
                return;
            }
            let owner = inner.owner.borrow_mut().take();
            let handler = inner.disconnected;
            let pending = Rc::downgrade(&inner);
            let app = app.clone();
            executor
                .spawn(async move {
                    if let Some(owner) = owner {
                        app.update(|cx| owner.dispatch_with(handler, close_owner::<C>, cx));
                    }
                    if let Some(inner) = pending.upgrade() {
                        inner.listener.borrow_mut().take();
                    }
                })
                .detach();
        });
        if let Ok(handle) = create_bridge(receive.as_ref(), disconnected.as_ref()) {
            self.inner.listener.replace(Some(Listener {
                handle,
                _receive: receive,
                _disconnected: disconnected,
            }));
            if self.inner.owner.borrow().is_some() {
                if let Some(listener) = self.inner.listener.borrow().as_ref() {
                    mark_ready(&listener.handle);
                }
            }
        }
    }
    pub fn bind_owner(&self, owner: OwnerAction<C>) {
        if self.inner.closed.get() {
            return;
        }
        self.inner.owner.replace(Some(owner));
        if let Some(listener) = self.inner.listener.borrow().as_ref() {
            mark_ready(&listener.handle);
        }
    }
    pub fn close(&self) {
        self.inner.closed.set(true);
        self.inner.owner.borrow_mut().take();
        self.inner.listener.borrow_mut().take();
    }
}
impl<C: 'static> Inner<C> {
    fn send(&self, reply: Reply) {
        {
            if let Some(listener) = self.listener.borrow().as_ref() {
                post_reply(
                    &listener.handle,
                    &reply.encode(),
                    if reply.error.is_none() {
                        reply.session.as_deref().unwrap_or("")
                    } else {
                        ""
                    },
                );
            }
        }
        if reply.terminal {
            self.closed.set(true);
            self.owner.borrow_mut().take();
            self.listener.borrow_mut().take();
        }
    }
}

fn close_owner<C>(owner: &mut C, handler: fn(&mut C) -> bool) -> ((), bool) {
    ((), handler(owner))
}
