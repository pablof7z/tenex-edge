use std::sync::Mutex;

use crate::session_host::session_has_live_delivery_path;
use crate::state::{Session, Store};

pub(super) fn automatic_delivery(store: &Mutex<Store>, session: Option<&Session>) -> bool {
    session.is_some_and(|session| {
        let store = store.lock().expect("store mutex poisoned");
        session_has_live_delivery_path(&store, session)
    })
}
