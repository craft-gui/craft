use craft::animation::animation::{Animation, KeyFrame, TimingFunction};
use craft::components::Context;
use craft::style::{StyleProperty, Unit};
use craft::{components::{Component, ComponentSpecification}, elements::{Container, ElementStyles}, palette, style::{Display, FlexDirection}};
use std::time::Duration;

#[derive(Default)]
pub struct AnimationsExample {
}

impl Component for AnimationsExample {
    type GlobalState = ();
    type Props = ();
    type Message = ();

    fn view(context: &mut Context<Self>) -> ComponentSpecification {
        let animation_examples = vec![
            Container::new()
                .background(palette::css::GRAY)
                .width("100px")
                .height("40px")
                .animation(
                    Animation::new(Duration::from_secs(5), TimingFunction::EaseOut)
                        .push(
                            KeyFrame::new(0.0)
                                .push(StyleProperty::Background(palette::css::BLACK))
                                .push(StyleProperty::Width(Unit::Px(20.0))),
                        )
                        .push(
                            KeyFrame::new(100.0)
                                .push(StyleProperty::Background(palette::css::RED))
                                .push(StyleProperty::Width(Unit::Px(200.0)))
                        )
                ),
        ];


        let mut container = Container::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .width("100%")
            .height("100%")
            .gap(20)
            ;

        for ani in animation_examples {
            container = container.push(ani)
        }

        container.component()
    }
}

#[allow(unused)]
#[cfg(not(target_os = "android"))]
fn main() {
    use craft::CraftOptions;
    util::setup_logging();
    craft::craft_main(AnimationsExample::component(), (), CraftOptions::basic("Counter"));
}
