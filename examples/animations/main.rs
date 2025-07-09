use craft::animation::animation::{Animation, KeyFrame, LoopAmount, TimingFunction};
use craft::components::Context;
use craft::elements::Text;
use craft::geometry::TrblRectangle;
use craft::style::{Position, StyleProperty, Unit};
use craft::{components::{Component, ComponentSpecification}, elements::{Container, ElementStyles}, palette, style::{Display, FlexDirection}};
use std::time::Duration;

#[derive(Default)]
pub struct AnimationsExample {
}

impl Component for AnimationsExample {
    type GlobalState = ();
    type Props = ();
    type Message = ();

    fn view(_context: &mut Context<Self>) -> ComponentSpecification {

        let growing_animation = Animation::new("growing_animation".to_string(), Duration::from_secs(5), TimingFunction::EaseOut)
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
                    .push(StyleProperty::Height(Unit::Px(100.0)))
            )
            .loop_amount(LoopAmount::Fixed(3))
            ;

        let moving_animation = Animation::new("moving_animation".to_string(), Duration::from_secs(5), TimingFunction::Ease)
            .push(
                KeyFrame::new(0.0)
                    .push(StyleProperty::Background(palette::css::BLUE))
                    .push(StyleProperty::Inset(TrblRectangle::new(Unit::Px(100.0), Unit::Px(0.0), Unit::Px(0.0), Unit::Px(0.0))))
            )
            .push(
                KeyFrame::new(50.0)
                    .push(StyleProperty::Background(palette::css::MAGENTA))
                    .push(StyleProperty::Inset(TrblRectangle::new(Unit::Px(150.0), Unit::Px(0.0), Unit::Px(0.0), Unit::Px(250.0))))
            )
            .push(
                KeyFrame::new(100.0)
                    .push(StyleProperty::Background(palette::css::YELLOW))
                    .push(StyleProperty::Inset(TrblRectangle::new(100.into(), 0.into(), 0.into(), 0.into())))
            )
            .loop_amount(LoopAmount::Infinite)
            ;

        let text_animation = Animation::new("text_animation".to_string(), Duration::from_secs(5), TimingFunction::Ease)
            .push(
                KeyFrame::new(0.0)
                    .push(StyleProperty::Background(palette::css::RED))
                    .push(StyleProperty::Color(palette::css::BLUE))
                    .push(StyleProperty::FontSize(20.0))
            )
            .push(
                KeyFrame::new(100.0)
                    .push(StyleProperty::Background(palette::css::YELLOW))
                    .push(StyleProperty::Color(palette::css::BLUE_VIOLET))
                    .push(StyleProperty::FontSize(40.0))
            )
            .loop_amount(LoopAmount::Infinite)
            ;

        let animation_examples: Vec<ComponentSpecification> = vec![
            Container::new()
                .background(palette::css::GRAY)
                .width("100px")
                .height("40px")
                .push_animation(growing_animation)
                .component(),

            Container::new()
            .push(
                Container::new()
                    .inset(Unit::Px(100.0), Unit::Px(0.0), Unit::Px(0.0), Unit::Px(0.0))
                    .background(palette::css::BLUE)
                    .position(Position::Absolute)
                    .width("40px")
                    .height("40px")
                    .push_animation(moving_animation)
            ).component(),

            Text::new("Why, Hello!")
                .push_animation(text_animation)
                .component()
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
    craft::craft_main(AnimationsExample::component(), (), CraftOptions::basic("Animations"));
}
