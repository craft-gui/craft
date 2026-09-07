use crate::style::{AlignContent, AlignItems, AlignSelf, JustifyContent};

impl From<AlignItems> for gummy::AlignItems {
    fn from(value: AlignItems) -> Self {
        match value {
            AlignItems::Normal => Self::Normal,
            AlignItems::Start => Self::Start,
            AlignItems::End => Self::End,
            AlignItems::FlexStart => Self::FlexStart,
            AlignItems::FlexEnd => Self::FlexEnd,
            AlignItems::SelfStart => Self::SelfStart,
            AlignItems::SelfEnd => Self::SelfEnd,
            AlignItems::Center => Self::Center,
            AlignItems::Baseline => Self::Baseline,
            AlignItems::Stretch => Self::Stretch,
            AlignItems::SafeStart => Self::SafeStart,
            AlignItems::SafeEnd => Self::SafeEnd,
            AlignItems::SafeFlexStart => Self::SafeFlexStart,
            AlignItems::SafeFlexEnd => Self::SafeFlexEnd,
            AlignItems::SafeSelfStart => Self::SafeSelfStart,
            AlignItems::SafeSelfEnd => Self::SafeSelfEnd,
            AlignItems::SafeCenter => Self::SafeCenter,
        }
    }
}

impl From<AlignSelf> for gummy::AlignSelf {
    fn from(value: AlignSelf) -> Self {
        match value {
            AlignSelf::Auto => Self::Auto,
            AlignSelf::Normal => Self::Normal,
            AlignSelf::Start => Self::Start,
            AlignSelf::End => Self::End,
            AlignSelf::FlexStart => Self::FlexStart,
            AlignSelf::FlexEnd => Self::FlexEnd,
            AlignSelf::SelfStart => Self::SelfStart,
            AlignSelf::SelfEnd => Self::SelfEnd,
            AlignSelf::Center => Self::Center,
            AlignSelf::Baseline => Self::Baseline,
            AlignSelf::Stretch => Self::Stretch,
            AlignSelf::SafeStart => Self::SafeStart,
            AlignSelf::SafeEnd => Self::SafeEnd,
            AlignSelf::SafeFlexStart => Self::SafeFlexStart,
            AlignSelf::SafeFlexEnd => Self::SafeFlexEnd,
            AlignSelf::SafeSelfStart => Self::SafeSelfStart,
            AlignSelf::SafeSelfEnd => Self::SafeSelfEnd,
            AlignSelf::SafeCenter => Self::SafeCenter,
        }
    }
}

impl From<AlignContent> for gummy::AlignContent {
    fn from(value: AlignContent) -> Self {
        match value {
            AlignContent::Normal => Self::Normal,
            AlignContent::Start => Self::Start,
            AlignContent::End => Self::End,
            AlignContent::FlexStart => Self::FlexStart,
            AlignContent::FlexEnd => Self::FlexEnd,
            AlignContent::Center => Self::Center,
            AlignContent::Stretch => Self::Stretch,
            AlignContent::SpaceBetween => Self::SpaceBetween,
            AlignContent::SpaceEvenly => Self::SpaceEvenly,
            AlignContent::SpaceAround => Self::SpaceAround,
            AlignContent::SafeStart => Self::SafeStart,
            AlignContent::SafeEnd => Self::SafeEnd,
            AlignContent::SafeFlexStart => Self::SafeFlexStart,
            AlignContent::SafeFlexEnd => Self::SafeFlexEnd,
            AlignContent::SafeCenter => Self::SafeCenter,
        }
    }
}

impl From<JustifyContent> for gummy::JustifyContent {
    fn from(value: JustifyContent) -> Self {
        match value {
            JustifyContent::Normal => Self::Normal,
            JustifyContent::Start => Self::Start,
            JustifyContent::End => Self::End,
            JustifyContent::FlexStart => Self::FlexStart,
            JustifyContent::FlexEnd => Self::FlexEnd,
            JustifyContent::Center => Self::Center,
            JustifyContent::Stretch => Self::Stretch,
            JustifyContent::SpaceBetween => Self::SpaceBetween,
            JustifyContent::SpaceEvenly => Self::SpaceEvenly,
            JustifyContent::SpaceAround => Self::SpaceAround,
            JustifyContent::SafeStart => Self::SafeStart,
            JustifyContent::SafeEnd => Self::SafeEnd,
            JustifyContent::SafeFlexStart => Self::SafeFlexStart,
            JustifyContent::SafeFlexEnd => Self::SafeFlexEnd,
            JustifyContent::SafeCenter => Self::SafeCenter,
        }
    }
}
