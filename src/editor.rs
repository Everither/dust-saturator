use nih_plug::prelude::Editor;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::widgets::*;
use nih_plug_vizia::{assets, create_vizia_editor, ViziaState, ViziaTheming};
use std::sync::Arc;

use crate::DustSaturatorParams;

#[derive(Lens)]
struct Data {
    params: Arc<DustSaturatorParams>
}

impl Model for Data {}

pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (300, 235))
}

pub(crate) fn create(
    params: Arc<DustSaturatorParams>,
    editor_state: Arc<ViziaState>,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, ViziaTheming::Custom, move |cx, _| {
        assets::register_noto_sans_light(cx);
        assets::register_noto_sans_thin(cx);

        Data {
            params: params.clone(),
        }
        .build(cx);

        ResizeHandle::new(cx);

        VStack::new(cx, |cx| {
            Label::new(cx, "Dust Saturator")
                .font_family(vec![FamilyOwned::Name(String::from(
                    assets::NOTO_SANS_THIN,
                ))])
                .color(Color::white())
                .font_size(30.0)
                .height(Pixels(50.0))
                .child_top(Stretch(1.0))
                .child_bottom(Pixels(0.0));

            Label::new(cx, "Amount").color(Color::white()).child_top(Pixels(10.0));

            ParamSlider::new(cx, Data::params, |params| &params.amount)
            .color(Color::white())
            .border_color(Color::white());

            Label::new(cx, "Curve").color(Color::white()).child_top(Pixels(10.0));

            ParamSlider::new(cx, Data::params, |params| &params.curve)
            .color(Color::white())
            .border_color(Color::white());

            Label::new(cx, "Version: 1.0.0")
            .font_size(11.0)
            .child_top(Pixels(40.0))
            .color(Color::rgb(170, 170, 170));
        })
        .background_color(Color::rgb(100, 100, 100))
        .row_between(Pixels(0.0))
        .child_left(Stretch(1.0))
        .child_right(Stretch(1.0));
    })
}
