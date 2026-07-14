use trellis_core::{GraphResult, TransactionResult};

use crate::fabric_context::ViewInputs;
use crate::reconcile::labels::NodeLabels;

use super::{build_nodes, opts, HookContextReconciler};

pub(crate) struct HookContextPreview {
    pub result: TransactionResult<()>,
    pub labels: NodeLabels,
}

impl HookContextReconciler {
    pub(crate) fn preview_context(
        &mut self,
        pubkey: &str,
        cursor: i64,
        now: i64,
        inputs: ViewInputs,
    ) -> GraphResult<HookContextPreview> {
        let mut labels = self.labels.clone();
        let mut tx = self.graph.begin_transaction_with_options(opts())?;
        let nodes = match self.nodes.as_ref() {
            Some(nodes) => nodes.clone(),
            None => build_nodes(&mut tx, pubkey, &mut labels)?,
        };
        tx.set_input(nodes.cursor, cursor)?;
        tx.set_input(nodes.now, now)?;
        tx.set_input(nodes.meta, inputs.meta)?;
        tx.set_input(nodes.members, inputs.members)?;
        tx.set_input(nodes.presence, inputs.presence)?;
        tx.set_input(nodes.messages, inputs.messages)?;
        let result = tx.preview()?;
        Ok(HookContextPreview { result, labels })
    }
}
