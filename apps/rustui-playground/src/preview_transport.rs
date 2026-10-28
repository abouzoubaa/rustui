//! Consumer-owned bounded transport over the existing public preview hosts.
use rustui::preview::*;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeSet;

pub(crate) const PROTOCOL: &str = "rustui.preview.postmessage";
pub(crate) const VERSION: u32 = 1;
pub(crate) const CONSUMER: &str = "rustui.playground.browser-preview-host";
pub(crate) const MAX_MESSAGE_BYTES: usize = 1_048_576;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Stamp {
    pub preview_id: PreviewId,
    pub generation: String,
    pub revision: String,
}
impl Stamp {
    fn preview(stamp: &PreviewStamp) -> Self {
        Self {
            preview_id: stamp.preview_id.clone(),
            generation: stamp.generation.to_string(),
            revision: stamp.revision.to_string(),
        }
    }
    fn composition(stamp: &CompositionStamp) -> Self {
        Self {
            preview_id: stamp.preview_id.clone(),
            generation: stamp.generation.to_string(),
            revision: stamp.revision.to_string(),
        }
    }
    fn as_preview(&self) -> Option<PreviewStamp> {
        Some(PreviewStamp {
            preview_id: self.preview_id.clone(),
            generation: decimal(&self.generation)?,
            revision: decimal(&self.revision)?,
        })
    }
    fn as_composition(&self) -> Option<CompositionStamp> {
        let stamp = self.as_preview()?;
        Some(CompositionStamp {
            preview_id: stamp.preview_id,
            generation: stamp.generation,
            revision: stamp.revision,
        })
    }
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Request {
    protocol: String,
    version: u32,
    client: String,
    pub request: String,
    pub session: Option<String>,
    pub sequence: String,
    command: Command,
}
impl Request {
    pub fn is_connect(&self) -> bool {
        matches!(&self.command, Command::Connect { .. })
    }
}
pub(crate) struct Delivery {
    pub request: Request,
    pub fresh: String,
}
#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Command {
    Connect {
        nonce: String,
    },
    Capabilities,
    Catalog,
    Mount {
        source: String,
    },
    EditProperties {
        stamp: Stamp,
        updates_source: String,
    },
    EditComposition {
        stamp: Stamp,
        edits_source: String,
    },
    Signal {
        stamp: Stamp,
        node: Option<NodeId>,
        event: EventId,
    },
    Export {
        stamp: Stamp,
    },
    Disconnect {
        stamp: Option<Stamp>,
    },
}
#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Edit {
    SetProperty {
        node: NodeId,
        property: PropertyId,
        value: PropertyValue,
    },
    Insert {
        parent: NodeId,
        slot: SlotId,
        node: NodeId,
        value: CompositionNode,
    },
    RemoveSubtree {
        node: NodeId,
    },
    Move {
        node: NodeId,
        parent: NodeId,
        slot: SlotId,
    },
}
impl From<Edit> for CompositionEdit {
    fn from(value: Edit) -> Self {
        match value {
            Edit::SetProperty {
                node,
                property,
                value,
            } => Self::SetProperty {
                node,
                property,
                value,
            },
            Edit::Insert {
                parent,
                slot,
                node,
                value,
            } => Self::Insert {
                parent,
                slot,
                node,
                value,
            },
            Edit::RemoveSubtree { node } => Self::RemoveSubtree { node },
            Edit::Move { node, parent, slot } => Self::Move { node, parent, slot },
        }
    }
}
#[derive(Debug, Serialize)]
pub(crate) struct Reply {
    #[serde(skip)]
    pub terminal: bool,
    protocol: &'static str,
    version: u32,
    pub request: String,
    pub session: Option<String>,
    sequence: String,
    pub result: Option<Value>,
    pub error: Option<&'static str>,
}
impl Reply {
    pub fn failure(request: &Request, code: &'static str) -> Self {
        Self::error(request, code)
    }
    pub fn unavailable(request: &Request) -> Self {
        Self::error(request, "owner_unavailable")
    }
    fn error(request: &Request, code: &'static str) -> Self {
        Self {
            terminal: false,
            protocol: PROTOCOL,
            version: VERSION,
            request: request.request.clone(),
            session: request.session.clone(),
            sequence: request.sequence.clone(),
            result: None,
            error: Some(code),
        }
    }
    pub fn malformed(code: &'static str) -> Self {
        Self {
            terminal: false,
            protocol: PROTOCOL,
            version: VERSION,
            request: String::new(),
            session: None,
            sequence: String::new(),
            result: None,
            error: Some(code),
        }
    }
    pub fn encode(&self) -> String {
        let wire = serde_json::to_string(self).expect("transport reply contains serializable data");
        if wire.len() <= MAX_MESSAGE_BYTES {
            wire
        } else {
            serde_json::to_string(&Self {
                result: None,
                error: Some("response_too_large"),
                ..Self {
                    terminal: false,
                    protocol: self.protocol,
                    version: self.version,
                    request: self.request.clone(),
                    session: self.session.clone(),
                    sequence: self.sequence.clone(),
                    result: None,
                    error: None,
                }
            })
            .unwrap()
        }
    }
}
pub(crate) fn decode(wire: &str) -> Result<Request, &'static str> {
    if wire.len() > MAX_MESSAGE_BYTES {
        return Err("message_too_large");
    }
    let request: Request = serde_json::from_str(wire).map_err(|_| "malformed_request")?;
    if !identity(&request.request)
        || !identity(&request.client)
        || decimal(&request.sequence).is_none()
    {
        return Err("invalid_identity");
    }
    if request
        .session
        .as_ref()
        .is_some_and(|session| !token(session))
    {
        return Err("invalid_session");
    }
    Ok(request)
}
fn identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|v| v.is_ascii_alphanumeric() || b"._:-".contains(&v))
}
fn token(value: &str) -> bool {
    value.len() == 32
        && value
            .bytes()
            .all(|v| v.is_ascii_hexdigit() && !v.is_ascii_uppercase())
}
fn decimal(value: &str) -> Option<u64> {
    if value.is_empty()
        || (value.len() > 1 && value.starts_with('0'))
        || !value.bytes().all(|v| v.is_ascii_digit())
    {
        None
    } else {
        value.parse().ok()
    }
}

pub(crate) enum Host<'a, C> {
    Preview(&'a mut PreviewHost<C>),
    Composition(&'a mut CompositionHost<C>),
}
impl<C> Host<'_, C> {
    fn catalog(&self) -> &ComponentCatalog {
        match self {
            Self::Preview(host) => host.catalog(),
            Self::Composition(host) => host.catalog(),
        }
    }
    fn stamp(&self) -> Option<Stamp> {
        match self {
            Self::Preview(host) => host.snapshot().map(|s| Stamp::preview(&s.stamp)),
            Self::Composition(host) => host.stamp().map(Stamp::composition),
        }
    }
    fn capabilities(&self, actions: &BTreeSet<ActionRef>) -> Value {
        let kind = match self {
            Self::Preview(_) => "preview",
            Self::Composition(_) => "composition",
        };
        let slots = match self {
            Self::Preview(_) => Value::Null,
            Self::Composition(host) => {
                Value::String(serde_json::to_string(host.composition_slots()).unwrap())
            }
        };
        let allowed: Vec<_> = actions.intersection(&self.catalog().actions).collect();
        json!({"profile":kind,"commands":if kind=="preview"{vec!["catalog","capabilities","mount","edit_properties","signal","export","disconnect"]}else{vec!["catalog","capabilities","mount","edit_composition","signal","export","disconnect"]},"preview_format":"rustui.preview","preview_version":1,"catalog_format":self.catalog().format,"catalog_version":self.catalog().format_version,"composition_format":COMPOSITION_FORMAT,"composition_version":COMPOSITION_FORMAT_VERSION,"slots_source":slots,"allowed_actions":allowed,"message_bytes":MAX_MESSAGE_BYTES,"pending_requests":1,"stamp":self.stamp()})
    }
    fn unmount(&mut self) {
        match self {
            Self::Preview(host) => {
                if let Some(stamp) = host.snapshot().map(|s| s.stamp.clone()) {
                    let _ = host.teardown(&stamp);
                }
            }
            Self::Composition(host) => host.unmount(),
        }
    }
    fn mount(&mut self, source: &str) -> Result<Stamp, &'static str> {
        match self {
            Self::Preview(host) => {
                let d = decode_preview(source, host.catalog()).map_err(|_| "invalid_document")?;
                host.mount(d.document)
                    .map(|s| Stamp::preview(&s))
                    .map_err(|_| "invalid_document")
            }
            Self::Composition(host) => host
                .mount_source(source)
                .map(|s| Stamp::composition(&s))
                .map_err(|_| "invalid_document"),
        }
    }
    fn export(&self) -> Result<String, &'static str> {
        let wire = match self {
            Self::Preview(host) => host
                .snapshot()
                .map(|s| encode_preview(&s.preview.document, host.catalog()))
                .transpose(),
            Self::Composition(host) => host.encode(),
        }
        .map_err(|_| "invalid_document")?
        .ok_or("not_mounted")?;
        Ok(wire)
    }
}

#[derive(Default)]
pub(crate) struct Session {
    connection: Option<Connection>,
}
struct Connection {
    token: String,
    nonce: String,
    next: u64,
}
pub(crate) struct Outcome {
    pub reply: Reply,
    pub action: Option<ActionRef>,
    pub changed: bool,
}
impl Session {
    pub fn close<C>(&mut self, mut host: Host<'_, C>) {
        host.unmount();
        self.connection = None;
    }
    pub fn handle<C>(
        &mut self,
        request: Request,
        mut host: Host<'_, C>,
        actions: &BTreeSet<ActionRef>,
        fresh: &str,
    ) -> Outcome {
        let error = |code| Outcome {
            reply: Reply::error(&request, code),
            action: None,
            changed: false,
        };
        if request.protocol != PROTOCOL || request.version != VERSION {
            return error("unsupported_version");
        }
        if request.client != CONSUMER {
            return error("unsupported_consumer");
        }
        if let Command::Connect { nonce } = &request.command {
            if request.session.is_some() || request.sequence != "0" || !token(nonce) {
                return error("invalid_connect");
            }
            let same = self.connection.as_ref().is_some_and(|c| c.nonce == *nonce);
            if !same {
                if !token(fresh) || self.connection.as_ref().is_some_and(|c| c.token == fresh) {
                    return error("session_rotation_failed");
                }
                host.unmount();
                self.connection = Some(Connection {
                    token: fresh.into(),
                    nonce: nonce.clone(),
                    next: 1,
                });
            }
            let session = self.connection.as_ref().unwrap().token.clone();
            let mut capabilities = host.capabilities(actions);
            capabilities["nonce"] = Value::String(nonce.clone());
            return Outcome {
                reply: Reply {
                    terminal: false,
                    protocol: PROTOCOL,
                    version: VERSION,
                    request: request.request,
                    session: Some(session),
                    sequence: request.sequence,
                    result: Some(capabilities),
                    error: None,
                },
                action: None,
                changed: !same,
            };
        }
        let Some(connection) = self.connection.as_mut() else {
            return error("disconnected");
        };
        if request.session.as_deref() != Some(connection.token.as_str()) {
            return error("stale_session");
        }
        let sequence = decimal(&request.sequence).unwrap();
        if sequence != connection.next {
            return error("stale_sequence");
        }
        let Some(next) = sequence.checked_add(1) else {
            return error("sequence_exhausted");
        };
        connection.next = next;
        let current = host.stamp();
        let expected = match &request.command {
            Command::EditProperties { stamp, .. }
            | Command::EditComposition { stamp, .. }
            | Command::Signal { stamp, .. }
            | Command::Export { stamp } => Some(Some(stamp)),
            Command::Disconnect { stamp } => Some(stamp.as_ref()),
            _ => None,
        };
        if expected.is_some_and(|stamp| stamp != current.as_ref()) {
            return error("stale_stamp");
        }
        let disconnect = matches!(&request.command, Command::Disconnect { .. });
        let mut action = None;
        let mut changed = false;
        let result: Result<Value, &'static str> = (|| match request.command {
            Command::Connect { .. } => unreachable!(),
            Command::Capabilities => Ok(host.capabilities(actions)),
            Command::Catalog => {
                let wire = encode_catalog(host.catalog()).map_err(|_| "invalid_catalog")?;
                Ok(json!({"source":wire}))
            }
            Command::Mount { source } => {
                let stamp = host.mount(&source)?;
                changed = true;
                Ok(json!({"stamp":stamp}))
            }
            Command::EditProperties {
                stamp,
                updates_source,
            } => match &mut host {
                Host::Preview(host) => {
                    let updates: Vec<(PropertyId, PropertyValue)> =
                        serde_json::from_str(&updates_source).map_err(|_| "invalid_edit")?;
                    if updates.len() > 256 {
                        return Err("edit_limit");
                    }
                    let stamp = host
                        .edit(&stamp.as_preview().ok_or("invalid_stamp")?, updates)
                        .map_err(|_| "invalid_edit")?;
                    let next = Stamp::preview(&stamp);
                    changed = current.as_ref() != Some(&next);
                    Ok(json!({"stamp":next}))
                }
                Host::Composition(_) => Err("unsupported_operation"),
            },
            Command::EditComposition {
                stamp,
                edits_source,
            } => match &mut host {
                Host::Composition(host) => {
                    let edits: Vec<Edit> =
                        serde_json::from_str(&edits_source).map_err(|_| "invalid_edit")?;
                    if edits.len() > MAX_COMPOSITION_EDITS {
                        return Err("edit_limit");
                    }
                    let edits: Vec<_> = edits.into_iter().map(CompositionEdit::from).collect();
                    let stamp = host
                        .edit(&stamp.as_composition().ok_or("invalid_stamp")?, &edits)
                        .map_err(|_| "invalid_edit")?;
                    let next = Stamp::composition(&stamp);
                    changed = current.as_ref() != Some(&next);
                    Ok(json!({"stamp":next}))
                }
                Host::Preview(_) => Err("unsupported_operation"),
            },
            Command::Signal { stamp, node, event } => {
                let target = match &mut host {
                    Host::Preview(host) => {
                        if node.is_some() {
                            return Err("invalid_signal");
                        }
                        host.dispatch(&PreviewSignal {
                            stamp: stamp.as_preview().ok_or("invalid_stamp")?,
                            event,
                        })
                    }
                    Host::Composition(host) => host.dispatch(&CompositionSignal {
                        stamp: stamp.as_composition().ok_or("invalid_stamp")?,
                        node: node.ok_or("invalid_signal")?,
                        event,
                    }),
                }
                .map_err(|_| "invalid_signal")?;
                if !actions.contains(&target) {
                    return Err("action_denied");
                }
                action = Some(target.clone());
                Ok(json!({"action":target}))
            }
            Command::Export { .. } => Ok(json!({"stamp":current,"source":host.export()?})),
            Command::Disconnect { .. } => {
                host.unmount();
                self.connection = None;
                changed = true;
                Ok(json!({"disconnected":true}))
            }
        })();
        let (result, error_code) = match result {
            Ok(value) => (Some(value), None),
            Err(code) => (None, Some(code)),
        };
        Outcome {
            reply: Reply {
                protocol: PROTOCOL,
                version: VERSION,
                request: request.request,
                session: request.session,
                sequence: request.sequence,
                result,
                error: error_code,
                terminal: disconnect && error_code.is_none(),
            },
            action,
            changed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustui::view::{AnyView, EmptyView};
    use std::collections::BTreeMap;
    fn registry() -> PreviewRegistry<()> {
        let mut registry = PreviewRegistry::new();
        for name in ["probe", "custom-consumer-component"] {
            registry
                .register(
                    ComponentSchema {
                        id: ComponentId::new(name).unwrap(),
                        version: 1,
                        display_name: name.into(),
                        properties: BTreeMap::from([(
                            PropertyId::new("label").unwrap(),
                            PropertySchema {
                                kind: PropertyKind::String,
                                default: Some(PropertyValue::String("default".into())),
                                required: false,
                                editable: true,
                                minimum: None,
                                maximum: None,
                                min_length: None,
                                max_length: Some(80),
                                allowed_values: vec![],
                            },
                        )]),
                        events: BTreeMap::from([(
                            EventId::new("activate").unwrap(),
                            EventSchema {
                                display_name: "Activate".into(),
                                payload: BTreeMap::new(),
                            },
                        )]),
                    },
                    |_, _| AnyView::new(EmptyView),
                )
                .unwrap();
        }
        registry
            .register_action(ActionRef::new("allowed").unwrap())
            .unwrap();
        registry
            .register_composable(
                ComponentSchema {
                    id: ComponentId::new("column").unwrap(),
                    version: 1,
                    display_name: "Column".into(),
                    properties: BTreeMap::new(),
                    events: BTreeMap::new(),
                },
                BTreeMap::from([(
                    SlotId::new("body").unwrap(),
                    SlotSchema {
                        allowed_children: BTreeSet::from([ComponentId::new("probe").unwrap()]),
                        min_children: 1,
                        max_children: 8,
                    },
                )]),
                |_, _, _| AnyView::new(EmptyView),
            )
            .unwrap();
        registry
    }
    fn wire(session: Option<&str>, sequence: u64, command: Value) -> Request {
        decode(&json!({"protocol":PROTOCOL,"version":1,"client":CONSUMER,"request":format!("r{sequence}"),"session":session,"sequence":sequence.to_string(),"command":command}).to_string()).unwrap()
    }
    fn connect(session: &mut Session, host: Host<'_, ()>) -> String {
        session
            .handle(
                wire(None, 0, json!({"kind":"connect","nonce":"a".repeat(32)})),
                host,
                &BTreeSet::new(),
                &"b".repeat(32),
            )
            .reply
            .session
            .unwrap()
    }
    fn document(component: &str) -> PreviewDocument {
        let mut document = PreviewDocument::new(
            PreviewId::new("external-preview").unwrap(),
            ComponentId::new(component).unwrap(),
            1,
        );
        document.actions.insert(
            EventId::new("activate").unwrap(),
            ActionRef::new("allowed").unwrap(),
        );
        document
    }
    #[test]
    fn strict_envelope_rejects_duplicates_unknowns_identity_and_size() {
        assert_eq!(
            decode(&" ".repeat(MAX_MESSAGE_BYTES + 1)).unwrap_err(),
            "message_too_large"
        );
        let base = json!({"protocol":PROTOCOL,"version":1,"client":CONSUMER,"request":"r1","session":null,"sequence":"0","command":{"kind":"catalog"}});
        let duplicate = base.to_string().replacen("{", "{\"version\":1,", 1);
        assert_eq!(decode(&duplicate).unwrap_err(), "malformed_request");
        let mut unknown = base.clone();
        unknown["surprise"] = json!(true);
        assert_eq!(
            decode(&unknown.to_string()).unwrap_err(),
            "malformed_request"
        );
        let mut sequence = base.clone();
        sequence["sequence"] = json!("01");
        assert_eq!(
            decode(&sequence.to_string()).unwrap_err(),
            "invalid_identity"
        );
        let mut session = base;
        session["session"] = json!("invalid");
        assert_eq!(decode(&session.to_string()).unwrap_err(), "invalid_session");
    }
    #[test]
    fn request_limits_count_utf8_bytes_and_accept_exact_boundaries() {
        let mut request = json!({
            "protocol": PROTOCOL,
            "version": VERSION,
            "client": CONSUMER,
            "request": "r".repeat(128),
            "session": null,
            "sequence": u64::MAX.to_string(),
            "command": {"kind": "mount", "source": "é".repeat(500_000)}
        });
        let mut wire = request.to_string();
        assert!(wire.len() < MAX_MESSAGE_BYTES);
        wire.push_str(&" ".repeat(MAX_MESSAGE_BYTES - wire.len()));
        assert!(decode(&wire).is_ok(), "the exact byte limit is accepted");
        wire.push(' ');
        assert_eq!(decode(&wire).unwrap_err(), "message_too_large");

        request["command"]["source"] = json!("");
        request["request"] = json!("r".repeat(129));
        assert_eq!(
            decode(&request.to_string()).unwrap_err(),
            "invalid_identity"
        );
        request["request"] = json!("r");
        request["sequence"] = json!("18446744073709551616");
        assert_eq!(
            decode(&request.to_string()).unwrap_err(),
            "invalid_identity"
        );
    }

    #[test]
    fn generic_catalog_mount_edit_export_and_replays_use_existing_host() {
        let mut host = PreviewHost::new(registry());
        let mut session = Session::default();
        let token = connect(&mut session, Host::Preview(&mut host));
        let catalog = session.handle(
            wire(Some(&token), 1, json!({"kind":"catalog"})),
            Host::Preview(&mut host),
            &BTreeSet::new(),
            "",
        );
        assert!(
            serde_json::from_str::<Value>(
                catalog.reply.result.unwrap()["source"].as_str().unwrap()
            )
            .unwrap()["components"]
                .get("custom-consumer-component")
                .is_some()
        );
        let source =
            encode_preview(&document("custom-consumer-component"), host.catalog()).unwrap();
        assert!(
            session
                .handle(
                    wire(Some(&token), 2, json!({"kind":"mount","source":source})),
                    Host::Preview(&mut host),
                    &BTreeSet::new(),
                    ""
                )
                .reply
                .error
                .is_none()
        );
        let old = Stamp::preview(&host.snapshot().unwrap().stamp);
        assert!(session.handle(wire(Some(&token),3,json!({"kind":"edit_properties","stamp":old,"updates_source":"[[\"label\",\"changed\"]]"})),Host::Preview(&mut host),&BTreeSet::new(),"").reply.error.is_none());
        let current = Stamp::preview(&host.snapshot().unwrap().stamp);
        assert_eq!(
            session
                .handle(
                    wire(Some(&token), 4, json!({"kind":"export","stamp":old})),
                    Host::Preview(&mut host),
                    &BTreeSet::new(),
                    ""
                )
                .reply
                .error,
            Some("stale_stamp")
        );
        let export = session.handle(
            wire(Some(&token), 5, json!({"kind":"export","stamp":current})),
            Host::Preview(&mut host),
            &BTreeSet::new(),
            "",
        );
        assert!(!export.changed);
        assert_eq!(
            serde_json::from_str::<Value>(export.reply.result.unwrap()["source"].as_str().unwrap())
                .unwrap()["properties"]["label"],
            "changed"
        );
        assert_eq!(
            session
                .handle(
                    wire(Some(&token), 5, json!({"kind":"catalog"})),
                    Host::Preview(&mut host),
                    &BTreeSet::new(),
                    ""
                )
                .reply
                .error,
            Some("stale_sequence")
        );
    }
    #[test]
    fn signals_default_deny_and_require_explicit_registered_action_policy() {
        let mut host = PreviewHost::new(registry());
        let mut session = Session::default();
        let token = connect(&mut session, Host::Preview(&mut host));
        let source = encode_preview(&document("probe"), host.catalog()).unwrap();
        session.handle(
            wire(Some(&token), 1, json!({"kind":"mount","source":source})),
            Host::Preview(&mut host),
            &BTreeSet::new(),
            "",
        );
        let stamp = Stamp::preview(&host.snapshot().unwrap().stamp);
        let denied = session.handle(
            wire(
                Some(&token),
                2,
                json!({"kind":"signal","stamp":stamp,"node":null,"event":"activate"}),
            ),
            Host::Preview(&mut host),
            &BTreeSet::new(),
            "",
        );
        assert_eq!(denied.reply.error, Some("action_denied"));
        assert!(denied.action.is_none());
        let allowed = BTreeSet::from([ActionRef::new("allowed").unwrap()]);
        let accepted = session.handle(
            wire(
                Some(&token),
                3,
                json!({"kind":"signal","stamp":stamp,"node":null,"event":"activate"}),
            ),
            Host::Preview(&mut host),
            &allowed,
            "",
        );
        assert_eq!(accepted.action, Some(ActionRef::new("allowed").unwrap()));
    }
    #[test]
    fn disconnect_and_new_session_revoke_stamps_and_old_session() {
        let mut host = PreviewHost::new(registry());
        let mut session = Session::default();
        let token = connect(&mut session, Host::Preview(&mut host));
        let source = encode_preview(&document("probe"), host.catalog()).unwrap();
        session.handle(
            wire(Some(&token), 1, json!({"kind":"mount","source":source})),
            Host::Preview(&mut host),
            &BTreeSet::new(),
            "",
        );
        let stamp = Stamp::preview(&host.snapshot().unwrap().stamp);
        assert!(
            session
                .handle(
                    wire(Some(&token), 2, json!({"kind":"disconnect","stamp":stamp})),
                    Host::Preview(&mut host),
                    &BTreeSet::new(),
                    ""
                )
                .reply
                .error
                .is_none()
        );
        assert!(host.snapshot().is_none());
        assert_eq!(
            session
                .handle(
                    wire(Some(&token), 3, json!({"kind":"catalog"})),
                    Host::Preview(&mut host),
                    &BTreeSet::new(),
                    ""
                )
                .reply
                .error,
            Some("disconnected")
        );
        let new = session
            .handle(
                wire(None, 0, json!({"kind":"connect","nonce":"c".repeat(32)})),
                Host::Preview(&mut host),
                &BTreeSet::new(),
                &"d".repeat(32),
            )
            .reply
            .session
            .unwrap();
        assert_ne!(new, token);
        assert_eq!(
            session
                .handle(
                    wire(Some(&token), 1, json!({"kind":"catalog"})),
                    Host::Preview(&mut host),
                    &BTreeSet::new(),
                    ""
                )
                .reply
                .error,
            Some("stale_session")
        );
        session.close(Host::Preview(&mut host));
        assert!(host.snapshot().is_none());
    }
    #[test]
    fn composition_metadata_and_all_existing_edit_operations_delegate_to_host() {
        let mut host = CompositionHost::new(registry());
        let mut session = Session::default();
        let token = connect(&mut session, Host::Composition(&mut host));
        assert!(
            host.composition_slots()
                .contains_key(&ComponentId::new("column").unwrap())
        );
        let preview_id = PreviewId::new("external-preview").unwrap();
        let root = NodeId::new("root").unwrap();
        let child = NodeId::new("child").unwrap();
        let root_preview =
            PreviewDocument::new(preview_id.clone(), ComponentId::new("column").unwrap(), 1);
        let document = CompositionDocument {
            format: COMPOSITION_FORMAT.into(),
            format_version: COMPOSITION_FORMAT_VERSION,
            preview_id,
            root: root.clone(),
            nodes: BTreeMap::from([
                (
                    root.clone(),
                    CompositionNode {
                        preview: root_preview,
                        slots: BTreeMap::from([(
                            SlotId::new("body").unwrap(),
                            vec![child.clone()],
                        )]),
                    },
                ),
                (
                    child.clone(),
                    CompositionNode {
                        preview: document("probe"),
                        slots: BTreeMap::new(),
                    },
                ),
            ]),
        };
        let source = serde_json::to_string(&document).unwrap();
        assert!(
            session
                .handle(
                    wire(Some(&token), 1, json!({"kind":"mount","source":source})),
                    Host::Composition(&mut host),
                    &BTreeSet::new(),
                    ""
                )
                .reply
                .error
                .is_none()
        );
        let inserted = CompositionNode {
            preview: super::tests::document("probe"),
            slots: BTreeMap::new(),
        };
        let edits = [
            json!({"kind":"set_property","node":"child","property":"label","value":"updated"}),
            json!({"kind":"insert","parent":"root","slot":"body","node":"second","value":inserted}),
            json!({"kind":"move","node":"second","parent":"root","slot":"body"}),
            json!({"kind":"remove_subtree","node":"child"}),
        ];
        for (index, edit) in edits.into_iter().enumerate() {
            let stamp = Stamp::composition(host.stamp().unwrap());
            assert!(
                session
                    .handle(
                        wire(
                            Some(&token),
                            index as u64 + 2,
                            json!({"kind":"edit_composition","stamp":stamp,"edits_source":serde_json::to_string(&vec![edit]).unwrap()})
                        ),
                        Host::Composition(&mut host),
                        &BTreeSet::new(),
                        ""
                    )
                    .reply
                    .error
                    .is_none()
            );
        }
        assert!(!host.document().unwrap().nodes.contains_key(&child));
        assert!(
            host.document()
                .unwrap()
                .nodes
                .contains_key(&NodeId::new("second").unwrap())
        );
    }
}
