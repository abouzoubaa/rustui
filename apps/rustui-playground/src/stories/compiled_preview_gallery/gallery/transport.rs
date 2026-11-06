impl CompiledPreviewGallery {
    fn current_status_presentation(&self) -> Option<&PreviewStatusPresentation> {
        self.status_presentation
            .as_ref()
            .filter(|p| p.status == self.status)
    }

    fn reference_owner_binding(&self) -> AnyView {
        #[cfg(target_family = "wasm")]
        if let Some(bridge) = self.reference_bridge.clone() {
            return AnyView::new(OwnerActionReader(move |owner: OwnerAction<Self>| {
                bridge.bind_owner(owner);
                EmptyView
            }));
        }
        AnyView::new(EmptyView)
    }

    fn reference_close(&mut self) -> bool {
        if let Some(mut session) = self.reference_session.take() {
            session.close(crate::preview_transport::Host::Preview(&mut self.host));
        }
        *self.stamp_lease.borrow_mut() = None;
        self.secure_value.clear();
        self.descriptor_json.clear();
        true
    }

    fn reference_increment(owner: &mut Self) -> bool {
        owner.activations += 1;
        true
    }

    fn reference_request(
        &mut self,
        delivery: crate::preview_transport::Delivery,
    ) -> (crate::preview_transport::Reply, bool) {
        use crate::preview_transport::{Host, Reply};
        let Some(mut session) = self.reference_session.take() else {
            return (Reply::unavailable(&delivery.request), false);
        };
        let policy: BTreeMap<ActionRef, fn(&mut Self) -> bool> = BTreeMap::from([(
            ActionRef::new("increment").unwrap(),
            Self::reference_increment as fn(&mut Self) -> bool,
        )]);
        let actions = policy.keys().cloned().collect();
        let before = self.host.snapshot().map(|snapshot| snapshot.stamp.clone());
        let mut outcome = session.handle(
            delivery.request,
            Host::Preview(&mut self.host),
            &actions,
            &delivery.fresh,
        );
        self.reference_session = Some(session);
        if let Some(action) = outcome.action.take() {
            outcome.changed |= policy
                .get(&action)
                .expect("transport checked action policy")(self);
        }
        let after = self.host.snapshot().map(|snapshot| snapshot.stamp.clone());
        if before != after {
            if before.as_ref().map(|s| s.generation) != after.as_ref().map(|s| s.generation) {
                self.secure_value.clear();
                self.stamp_lease = Rc::new(RefCell::new(after));
            } else {
                *self.stamp_lease.borrow_mut() = after;
            }
            self.refresh_number_drafts();
            self.descriptor_json = self
                .host
                .snapshot()
                .and_then(|snapshot| {
                    encode_preview(&snapshot.preview.document, self.host.catalog()).ok()
                })
                .unwrap_or_default();
        }
        if outcome.changed {
            self.status = "Browser reference request accepted".into();
        }
        (outcome.reply, outcome.changed)
    }
}
