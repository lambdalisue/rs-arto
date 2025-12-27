use dioxus::prelude::*;

/// Slider input component with numeric input and "Use Current" option
#[component]
pub fn SliderInput(
    value: f64,
    min: f64,
    max: f64,
    step: f64,
    unit: String,
    on_change: EventHandler<f64>,
    current_value: Option<f64>,
) -> Element {
    let handle_use_current = move |_| {
        if let Some(current) = current_value {
            on_change.call(current);
        }
    };

    let handle_number_input = move |evt: Event<FormData>| {
        if let Ok(new_value) = evt.value().parse::<f64>() {
            let clamped = new_value.clamp(min, max);
            on_change.call(clamped);
        }
    };

    rsx! {
        div {
            class: "slider-input",
            input {
                r#type: "range",
                min: "{min}",
                max: "{max}",
                step: "{step}",
                value: "{value}",
                oninput: move |evt| {
                    if let Ok(new_value) = evt.value().parse::<f64>() {
                        on_change.call(new_value);
                    }
                },
            }
            div {
                class: "slider-value-input",
                input {
                    r#type: "number",
                    min: "{min}",
                    max: "{max}",
                    step: "{step}",
                    value: "{value as i32}",
                    oninput: handle_number_input,
                }
                span { "{unit}" }
            }
            button {
                class: "use-current-button",
                disabled: current_value.is_none(),
                onclick: handle_use_current,
                "Use Current"
            }
        }
    }
}
