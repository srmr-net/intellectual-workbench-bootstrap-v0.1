//! Atomic controller publication. The invalidator uses the same mutex.
use std::sync::Mutex;
/// Keep validation, construction, installation and the caller's clone in one
/// critical section. Otherwise invalidation can miss a not-yet-published value.
pub fn get_or_try_create<T: Clone, E>(
    slot: &Mutex<Option<T>>,
    valid: impl Fn(&T) -> bool,
    on_invalidate: impl FnOnce(T),
    create: impl FnOnce() -> Result<T, E>,
) -> Result<T, E> {
    let mut guard = slot.lock().unwrap();
    if guard.as_ref().is_some_and(|value| !valid(value)) {
        on_invalidate(guard.take().unwrap());
    }
    if guard.is_none() {
        *guard = Some(create()?);
    }
    Ok(guard.as_ref().unwrap().clone())
}
