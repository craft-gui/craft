use craft_retained::elements::Element;
use craft_retained::events::ui_events::pointer::PointerButtonEvent;
use craft_retained::style::{Overflow, StyleProperty, Unit};
use craft_retained::{elements::{Button, Container, Text}, palette, rgb, style::{AlignItems, Display, FlexDirection, JustifyContent}, Color};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use craft_retained::animations::{Animation, KeyFrame, LoopAmount, TimingFunction};

#[derive(Default)]
pub struct Counter {
    count: i64,
}

impl Counter {

    fn decrement(&mut self) {
        self.count -= 1;
    }

    fn increment(&mut self) {
        self.count += 1;
    }

    fn count(&self) -> i64 {
        self.count
    }

}

#[allow(unused)]
#[cfg(not(target_os = "android"))]
fn main() {
    println!("Hello World");
    let root = Container::new();
    let body = Container::new();
    let root2 = root.clone();

    let button = Container::new();
    let inc = Text::new("Increment");
    inc.borrow_mut().set_selectable(false);
    button.borrow_mut().push(inc);

    let growing_animation = Animation::new("growing_animation", Duration::from_secs(5), TimingFunction::Ease)
        .push(
            KeyFrame::new(0.0)
                .push(StyleProperty::Background(palette::css::GREEN))
                .push(StyleProperty::Width(Unit::Percentage(10.0)))
                .push(StyleProperty::Height(Unit::Px(40.0))),
        )
        .push(
            KeyFrame::new(100.0)
                .push(StyleProperty::Background(palette::css::RED))
                .push(StyleProperty::Width(Unit::Percentage(80.0)))
                .push(StyleProperty::Height(Unit::Px(100.0))),
        )
        .loop_amount(LoopAmount::Infinite);

    //button.borrow_mut().style_mut().animations.push(growing_animation);

    button.borrow_mut().style_mut().set_background(Color::from_rgb8(255, 0, 0));
    //button.borrow_mut().style_mut().set_width(Unit::Px(100.0));
    //button.borrow_mut().style_mut().set_height(Unit::Px(100.0));
    //body.borrow_mut().element_data_mut().current_style_mut().set_border_radius([(20.0, 20.0); 4]);

    root.borrow_mut().push(body.clone());
    //body.borrow_mut().push(button.clone());

    body.borrow_mut().element_data_mut().current_style_mut().set_background(Color::from_rgb8(0, 255, 0));
    //body.borrow_mut().element_data_mut().current_style_mut().set_width(Unit::Px(100.0));
    //body.borrow_mut().element_data_mut().current_style_mut().set_height(Unit::Px(100.0));
    //body.borrow_mut().element_data_mut().current_style_mut().set_border_radius([(20.0, 20.0); 4]);

    let text = Text::new("Count: 0");

    let count = Rc::new(RefCell::new(Counter::default()));

    //body.borrow_mut().push(text.clone());

    let text2 = text.clone();

    button.borrow_mut().on_pointer_button_down(Rc::new(move |_, _| {
        /*let mut text = text.borrow_mut();
        let new_text = if text.text() == "foo" { "bar" } else { "foo" };
        text.set_text(new_text);*/
    }));

    button.borrow_mut().on_pointer_button_up(Rc::new(move |_, e| {
        if let Some(craft_retained::events::ui_events::pointer::PointerButton::Primary) = e.button {
            let mut count = count.borrow_mut();
            count.increment();
            text2.borrow_mut().set_text(&format!("Count: {}", count.count()));
        }
    }));

    let scroll = Container::new();
    scroll.borrow_mut().style_mut().set_overflow([Overflow::Visible, Overflow::Scroll]);
    scroll.borrow_mut().style_mut().set_background(Color::from_rgb8(0, 255, 0));
    scroll.borrow_mut().style_mut().set_width(Unit::Px(200.0));
    scroll.borrow_mut().style_mut().set_height(Unit::Px(200.0));

    let content = Container::new();

    content.borrow_mut().style_mut().set_background(Color::from_rgb8(0, 255, 255));
    content.borrow_mut().style_mut().set_width(Unit::Px(50.0));
    content.borrow_mut().style_mut().set_height(Unit::Px(500.0));

    scroll.borrow_mut().push(content);

    body.borrow_mut().push(scroll);

    use craft_retained::CraftOptions;
    util::setup_logging();
    craft_retained::craft_main(root2, CraftOptions::basic("Counter"));
}
