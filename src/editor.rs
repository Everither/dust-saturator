use nih_plug::prelude::Editor;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::widgets::*;
use nih_plug_vizia::{assets, create_vizia_editor, ViziaState, ViziaTheming};
use std::sync::Arc;

use crate::LinearInterpolatorParams;

#[derive(Lens)]
struct Data {
    params: Arc<LinearInterpolatorParams>
}

impl Model for Data {}

pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (300, 250))
}

pub(crate) fn create(
    params: Arc<LinearInterpolatorParams>,
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
            Label::new(cx, "Linear Interpolator")
                .font_family(vec![FamilyOwned::Name(String::from(
                    assets::NOTO_SANS_THIN,
                ))])
                .font_size(30.0)
                .height(Pixels(50.0))
                .child_top(Stretch(1.0))
                .child_bottom(Pixels(0.0));

            Label::new(cx, "Amount").child_top(Pixels(10.0));

            HStack::new(cx, |cx| {
                ParamSlider::new(cx, Data::params, |params| &params.amount).width(Pixels(128.0));
                ParamButton::new(cx,  Data::params, |params| &params.dither).width(Pixels(44.0));
            }).height(Pixels(40.0));

            Label::new(cx, "Tolerance");

            ParamSlider::new(cx, Data::params, |params| &params.tolerance).width(Pixels(172.0));
            

        })
        .row_between(Pixels(0.0))
        .child_left(Stretch(1.0))
        .child_right(Stretch(1.0));
    })
}
