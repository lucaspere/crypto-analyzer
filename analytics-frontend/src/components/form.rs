use crate::types::*;
use chrono::Utc;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct AnalyticsFormProps {
    pub on_submit: Callback<AnalyticsRequest>,
}

#[derive(Clone, PartialEq)]
pub struct AnalyticsRequest {
    pub symbol: String,
    pub analytics_type: AnalyticsType,
    pub time_range: TimeRange,
    pub window_size: u32,
    pub fast_period: u32,
    pub slow_period: u32,
    pub signal_period: u32,
}

#[function_component(AnalyticsForm)]
pub fn analytics_form(props: &AnalyticsFormProps) -> Html {
    let symbol = use_state(|| "BTCUSDT".to_string());
    let analytics_type = use_state(|| AnalyticsType::Vwap);
    let window_size = use_state(|| 20u32);
    let fast_period = use_state(|| 12u32);
    let slow_period = use_state(|| 26u32);
    let signal_period = use_state(|| 9u32);

    let on_submit = {
        let symbol = symbol.clone();
        let analytics_type = analytics_type.clone();
        let window_size = window_size.clone();
        let fast_period = fast_period.clone();
        let slow_period = slow_period.clone();
        let signal_period = signal_period.clone();
        let on_submit = props.on_submit.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let start_time = Utc::now() - chrono::Duration::minutes(5);
            let end_time = Utc::now();
            let request = AnalyticsRequest {
                symbol: (*symbol).clone(),
                analytics_type: (*analytics_type).clone(),
                time_range: TimeRange {
                    start: start_time,
                    end: end_time,
                },
                window_size: *window_size,
                fast_period: *fast_period,
                slow_period: *slow_period,
                signal_period: *signal_period,
            };
            on_submit.emit(request);
        })
    };

    let analytics_type_value = (*analytics_type).clone();

    html! {
        <form onsubmit={on_submit} class="bg-white shadow rounded-lg p-6">
            <h3 class="text-lg font-medium text-gray-900 mb-4">{"Analytics Request"}</h3>

            <div class="grid grid-cols-1 gap-4">
                <div>
                    <label class="block text-sm font-medium text-gray-700">{"Symbol"}</label>
                    <input
                        type="text"
                        value={(*symbol).clone()}
                        oninput={Callback::from(move |e: InputEvent| {
                            if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                                symbol.set(input.value());
                            }
                        })}
                        class="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500"
                        placeholder="BTCUSDT"
                    />
                </div>

                <div>
                    <label class="block text-sm font-medium text-gray-700">{"Analytics Type"}</label>
                    <select
                        value={analytics_type_value.as_str()}
                        onchange={Callback::from(move |e: Event| {
                            if let Some(select) = e.target_dyn_into::<web_sys::HtmlElement>() {
                                let new_type = match select.get_attribute("value").unwrap_or_default().as_str() {
                                    "SMA" => AnalyticsType::Sma,
                                    "MACD" => AnalyticsType::Macd,
                                    _ => AnalyticsType::Vwap,
                                };
                                analytics_type.set(new_type);
                            }
                        })}
                        class="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500"
                    >
                        <option value="VWAP">{"VWAP"}</option>
                        <option value="SMA">{"SMA"}</option>
                        <option value="MACD">{"MACD"}</option>
                    </select>
                </div>

                if analytics_type_value == AnalyticsType::Sma {
                    <div>
                        <label class="block text-sm font-medium text-gray-700">{"Window Size"}</label>
                        <input
                            type="number"
                            value={(*window_size).to_string()}
                            oninput={Callback::from(move |e: InputEvent| {
                                if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                                    if let Ok(value) = input.value().parse::<u32>() {
                                        window_size.set(value);
                                    }
                                }
                            })}
                            class="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500"
                            min="1"
                            max="100"
                        />
                    </div>
                }

                if analytics_type_value == AnalyticsType::Macd {
                    <div class="grid grid-cols-3 gap-4">
                        <div>
                            <label class="block text-sm font-medium text-gray-700">{"Fast Period"}</label>
                            <input
                                type="number"
                                value={(*fast_period).to_string()}
                                oninput={Callback::from(move |e: InputEvent| {
                                    if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                                        if let Ok(value) = input.value().parse::<u32>() {
                                            fast_period.set(value);
                                        }
                                    }
                                })}
                                class="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500"
                                min="1"
                                max="50"
                            />
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-gray-700">{"Slow Period"}</label>
                            <input
                                type="number"
                                value={(*slow_period).to_string()}
                                oninput={Callback::from(move |e: InputEvent| {
                                    if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                                        if let Ok(value) = input.value().parse::<u32>() {
                                            slow_period.set(value);
                                        }
                                    }
                                })}
                                class="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500"
                                min="1"
                                max="100"
                            />
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-gray-700">{"Signal Period"}</label>
                            <input
                                type="number"
                                value={(*signal_period).to_string()}
                                oninput={Callback::from(move |e: InputEvent| {
                                    if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                                        if let Ok(value) = input.value().parse::<u32>() {
                                            signal_period.set(value);
                                        }
                                    }
                                })}
                                class="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500"
                                min="1"
                                max="50"
                            />
                        </div>
                    </div>
                }

                <button
                    type="submit"
                    class="w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
                >
                    {"Get Analytics"}
                </button>
            </div>
        </form>
    }
}
