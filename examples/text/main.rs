use craft_retained::elements::{Container, Element, TextInput, Window};
use craft_retained::rgb;
use craft_retained::style::{AlignItems, Display, FlexDirection, JustifyContent, Overflow, Unit};

const LOREM_IPSUM: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. In dolor tortor, congue eu lacus eget, faucibus aliquet nunc. Suspendisse aliquet ullamcorper fermentum. Pellentesque eu nibh sit amet nisi maximus pulvinar quis eget lectus. Class aptent taciti sociosqu ad litora torquent per conubia nostra, per inceptos himenaeos. Curabitur efficitur metus maximus libero maximus pharetra. Sed fringilla ac velit nec hendrerit. Mauris vulputate ante non tempor iaculis. In a diam eros. Mauris vehicula, lacus rhoncus consequat laoreet, lectus ligula venenatis leo, a aliquam odio ex ut lorem. Phasellus quis fermentum erat, ut pellentesque diam. Suspendisse pulvinar eros magna, vel ultricies metus luctus at. Pellentesque consequat a magna ut cursus. Nunc nisl velit, maximus blandit mauris quis, convallis cursus leo. Donec ut ultrices dui, a efficitur sem. Cras ac diam non orci sagittis tincidunt. Etiam auctor ultrices leo vitae vestibulum.

Nunc malesuada eleifend magna eget sollicitudin. Phasellus non posuere justo. Ut viverra posuere molestie. Aenean diam orci, dignissim eu diam vel, viverra suscipit libero. Nam convallis sed arcu porttitor aliquet. Pellentesque urna risus, consectetur bibendum metus vel, laoreet fringilla sapien. Curabitur justo turpis, auctor vel quam quis, volutpat scelerisque magna. Proin ornare, turpis ac eleifend consequat, augue nulla interdum nulla, a scelerisque mi tortor a nulla. Aenean ut sollicitudin quam.

Ut ac magna dolor. Etiam tempor varius arcu. Sed sit amet convallis quam. Phasellus varius vestibulum condimentum. Sed felis nulla, vehicula tempor fringilla eget, cursus sit amet metus. Nunc imperdiet metus nec ante porttitor tristique. In venenatis tortor sed aliquam dignissim. Donec tempus mollis enim in volutpat. Nam tincidunt sed mauris ut facilisis. Pellentesque tempus dolor at maximus vehicula. Sed nec facilisis nibh. Praesent molestie porttitor sem scelerisque malesuada. In pulvinar malesuada elit, ut ultricies nunc faucibus vel.

Morbi tincidunt porta scelerisque. Etiam sodales, leo eget molestie imperdiet, libero enim ullamcorper erat, eget iaculis nisi purus a metus. Proin ante elit, eleifend at gravida vel, accumsan vitae felis. Nullam id dolor vel felis faucibus aliquam vitae et purus. Curabitur malesuada, sapien vitae rutrum imperdiet, orci lorem imperdiet metus, at consequat justo libero non dui. Nam ac orci turpis. Aenean tristique urna velit. Vivamus laoreet ex sed dapibus mollis. Integer ante lorem, tincidunt nec pulvinar eu, auctor vel sem. Nulla non ex vitae dui viverra ultrices. In quis est enim. Duis consectetur mauris tortor, non consectetur metus aliquam sit amet. Vestibulum auctor tincidunt luctus. Aliquam aliquet dictum diam, in volutpat nunc egestas eget. Praesent dui leo, posuere in fringilla quis, congue nec nunc.

Nunc tellus magna, varius eu ornare et, sodales hendrerit quam. Praesent nec magna finibus, elementum orci nec, facilisis nisi. Duis ligula mi, dapibus eget nibh a, posuere viverra ante. Aliquam efficitur mauris id quam faucibus, eget posuere turpis imperdiet. Nam vulputate sed urna vitae tincidunt. Nulla ligula urna, iaculis id urna sit amet, porta iaculis ligula. Maecenas volutpat odio at pretium commodo. Nullam faucibus efficitur neque, vitae elementum sem sollicitudin eu. Nullam rutrum nulla eu erat dignissim varius. ";

pub fn text() -> Container {
    Container::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .justify_content(Some(JustifyContent::Center))
        .align_items(Some(AlignItems::Center))
        .width(Unit::Percentage(100.0))
        .height(Unit::Percentage(100.0))
        .gap(Unit::Px(20.0), Unit::Px(20.0))
        .font_size(72.0)
        .color(rgb(50, 50, 50))
        .push(
            TextInput::new(LOREM_IPSUM)
                .overflow(Overflow::Visible, Overflow::Scroll)
                .width(Unit::Px(600.0))
                .height(Unit::Px(600.0))
                .display(Display::Block),
        )
}

pub fn main() {
    Window::new("Text").push(text());
    use craft_retained::CraftOptions;
    util::setup_logging();
    craft_retained::craft_main(CraftOptions::basic("text"));
}
