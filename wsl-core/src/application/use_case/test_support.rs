use std::sync::Mutex;

#[derive(Default)]
pub(crate) struct CallLog {
    calls: Mutex<Vec<String>>,
}

impl CallLog {
    pub(crate) fn record(&self, call: impl Into<String>) {
        self.calls
            .lock()
            .expect("call log mutex poisoned")
            .push(call.into());
    }

    pub(crate) fn calls(&self) -> Vec<String> {
        self.calls.lock().expect("call log mutex poisoned").clone()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.calls
            .lock()
            .expect("call log mutex poisoned")
            .is_empty()
    }
}
