use core::sync::atomic::*;

static KEY_PRESSED: AtomicU32 = AtomicU32::new(0);
static INCOMING_KEY: AtomicBool = AtomicBool::new(false);

pub fn read_key() -> Option<u32> {
    INCOMING_KEY.load(Ordering::Relaxed).then(|| {
        INCOMING_KEY.store(false, Ordering::Relaxed);
        KEY_PRESSED.load(Ordering::Relaxed)
    })
}

pub fn register_key(k: u32) {
    if !INCOMING_KEY.load(Ordering::Relaxed) {
        KEY_PRESSED.store(k, Ordering::Relaxed);
        INCOMING_KEY.store(true, Ordering::Relaxed);
    }
}
