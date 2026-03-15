//! Internal module for macro-generated code.
//! Not part of the public API.

use std::cell::RefCell;
use std::any::Any;

pub use crate::ui::reset_id_counter;

thread_local! {
    static APP_INSTANCE: RefCell<Option<Box<dyn Any>>> = const { RefCell::new(None) };
}

pub fn set_app_instance(app: Box<dyn Any>) {
    APP_INSTANCE.with(|cell| {
        *cell.borrow_mut() = Some(app);
    });
}

pub fn with_app<T: 'static, F, R>(f: F) -> Option<R>
where
    F: FnOnce(&T) -> R,
{
    APP_INSTANCE.with(|cell| {
        let borrow = cell.borrow();
        borrow.as_ref().and_then(|app| {
            app.downcast_ref::<T>().map(f)
        })
    })
}

pub fn with_app_mut<T: 'static, F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut T) -> R,
{
    APP_INSTANCE.with(|cell| {
        // Use try_borrow_mut to avoid panic if re-entered (e.g., tool handler
        // calling another tool). Returns None instead of panicking.
        let mut borrow = cell.try_borrow_mut().ok()?;
        borrow.as_mut().and_then(|app| {
            app.downcast_mut::<T>().map(f)
        })
    })
}

