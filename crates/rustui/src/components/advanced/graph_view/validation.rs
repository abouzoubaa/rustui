use super::*;

pub(super) fn identity(value: &str) -> Result<(), GraphError> {
    if value.trim().is_empty() {
        Err(GraphError::BlankIdentity)
    } else {
        Ok(())
    }
}
pub(super) fn label(value: &str) -> Result<(), GraphError> {
    if value.trim().is_empty() {
        Err(GraphError::BlankLabel)
    } else {
        Ok(())
    }
}

impl GraphNode {
    pub(super) fn validate(&self) -> Result<(), GraphError> {
        identity(&self.id)?;
        label(&self.label)?;
        CanvasRect::new(self.rect.x, self.rect.y, self.rect.width, self.rect.height)
            .map_err(|_| GraphError::InvalidGeometry)?;
        let mut ids = HashSet::new();
        for port in &self.ports {
            identity(&port.id)?;
            label(&port.label)?;
            if !ids.insert(&port.id) {
                return Err(GraphError::DuplicatePort);
            }
        }
        Ok(())
    }
}

impl GraphDocument {
    pub(super) fn validate(&self) -> Result<(), GraphError> {
        let mut nodes = HashMap::with_capacity(self.nodes.len());
        for node in &self.nodes {
            node.validate()?;
            if nodes.insert(&node.id, node).is_some() {
                return Err(GraphError::DuplicateNode);
            }
        }
        let endpoint = |endpoint: &GraphEndpoint| {
            let node = nodes.get(&endpoint.node).ok_or(GraphError::MissingNode)?;
            node.ports
                .iter()
                .find(|port| port.id == endpoint.port)
                .ok_or(GraphError::MissingPort)
        };
        let mut ids = HashSet::new();
        let mut endpoints = HashSet::new();
        for edge in &self.edges {
            identity(&edge.id)?;
            if !ids.insert(&edge.id) {
                return Err(GraphError::DuplicateEdge);
            }
            if !endpoints.insert((
                &edge.from.node,
                &edge.from.port,
                &edge.to.node,
                &edge.to.port,
            )) {
                return Err(GraphError::DuplicateConnection);
            }
            if endpoint(&edge.from)?.direction != GraphPortDirection::Output
                || endpoint(&edge.to)?.direction != GraphPortDirection::Input
            {
                return Err(GraphError::PortDirection);
            }
        }
        Ok(())
    }
    pub(super) fn validate_new_edge(&self, edge: &GraphEdge) -> Result<(), GraphError> {
        identity(&edge.id)?;
        if self.edges.iter().any(|existing| existing.id == edge.id) {
            return Err(GraphError::DuplicateEdge);
        }
        if self
            .edges
            .iter()
            .any(|existing| existing.from == edge.from && existing.to == edge.to)
        {
            return Err(GraphError::DuplicateConnection);
        }
        if self.endpoint(&edge.from)?.1.direction != GraphPortDirection::Output
            || self.endpoint(&edge.to)?.1.direction != GraphPortDirection::Input
        {
            return Err(GraphError::PortDirection);
        }
        Ok(())
    }
}
