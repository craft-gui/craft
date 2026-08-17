# Contributing

## Running Tests

### RetGui Integration Tests
```shell
cargo test -p retgui --tests --no-default-features --features integration_tests
```

## File Order

Pub use is first. Std imports should be first. Then alphabetically by create names.  Then crate imports. Then modules. Then pub modules. Then pub type allaises then type alliases. Then pub structs then structs. then trait impls. Then struct impls.

```rust
pub use retgui_primitives::Color;

use std::collections::VecDeque;
use std::rc::Rc;

use gummy::NodeId;
use winit::window::WindowId;

use crate::app::App;
use crate::elements::Element;

mod helpers;

pub mod widgets;

pub type PublicId = u64;

type EventQueue = VecDeque<u64>;

pub struct PublicState {
    id: PublicId,
}

struct InternalState {
    queue: EventQueue,
}

impl Default for PublicState {
    fn default() -> Self {
        Self { id: 0 }
    }
}

impl PublicState {
    pub fn id(&self) -> PublicId {
        self.id
    }
}
```
