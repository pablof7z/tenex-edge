use super::*;

impl DaemonState {
    pub(crate) fn drive_delivery_scan(
        &self,
        _trigger: &str,
        fact: crate::reconcile::DeliveryScanFact,
    ) -> Result<Vec<crate::reconcile::DeliveryEffect>> {
        crate::delivery_seam::drive(&self.store, fact)
    }
}
