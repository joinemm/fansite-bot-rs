use serenity_utils::menu::{next_page, prev_page, Control, MenuOptions};
use std::sync::Arc;

pub fn simple_options() -> MenuOptions {
    let controls = vec![
        Control::new('◀'.into(), Arc::new(|m, r| Box::pin(prev_page(m, r)))),
        Control::new('▶'.into(), Arc::new(|m, r| Box::pin(next_page(m, r)))),
    ];
    let options = MenuOptions {
        controls,
        timeout: 120.0,
        ..Default::default()
    };
    options
}
