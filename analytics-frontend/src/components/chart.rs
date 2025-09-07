use crate::types::*;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ChartProps {
    pub data: Vec<MovingAveragePoint>,
    pub title: String,
}

#[function_component(Chart)]
pub fn chart(props: &ChartProps) -> Html {
    html! {
        <div class="bg-white shadow rounded-lg p-6">
            <h3 class="text-lg font-medium text-gray-900 mb-4">{&props.title}</h3>
            <div class="h-64 flex items-end justify-between space-x-1">
                {props.data.iter().enumerate().map(|(i, point)| {
                    let height = (point.value / props.data.iter().map(|p| p.value).fold(0.0, f64::max) * 200.0) as u32;
                    html! {
                        <div key={i} class="bg-blue-500 rounded-t" style={format!("height: {}px; width: 4px;", height)}>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
            <div class="mt-4 text-sm text-gray-600">
                {"Latest: "}{format!("{:.2}", props.data.last().map(|p| p.value).unwrap_or(0.0))}
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct MacdChartProps {
    pub data: Vec<MacdPoint>,
    pub title: String,
}

#[function_component(MacdChart)]
pub fn macd_chart(props: &MacdChartProps) -> Html {
    html! {
        <div class="bg-white shadow rounded-lg p-6">
            <h3 class="text-lg font-medium text-gray-900 mb-4">{&props.title}</h3>
            <div class="space-y-4">
                <div class="h-32 flex items-end justify-between space-x-1">
                    {props.data.iter().enumerate().map(|(i, point)| {
                        let macd_height = (point.macd_line.abs() / 10.0 * 50.0) as u32;
                        let signal_height = (point.signal_line.abs() / 10.0 * 50.0) as u32;
                        html! {
                            <div key={i} class="flex flex-col items-center space-y-1">
                                <div class="bg-blue-500 rounded-t" style={format!("height: {}px; width: 2px;", macd_height)}></div>
                                <div class="bg-red-500 rounded-t" style={format!("height: {}px; width: 2px;", signal_height)}></div>
                            </div>
                        }
                    }).collect::<Vec<_>>()}
                </div>
                <div class="h-16 flex items-end justify-between space-x-1">
                    {props.data.iter().enumerate().map(|(i, point)| {
                        let histogram_height = (point.histogram.abs() / 5.0 * 30.0) as u32;
                        let color = if point.histogram >= 0.0 { "bg-green-500" } else { "bg-red-500" };
                        html! {
                            <div key={i} class={format!("{} rounded-t", color)} style={format!("height: {}px; width: 2px;", histogram_height)}>
                            </div>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            </div>
            <div class="mt-4 text-sm text-gray-600">
                <div>{"MACD: "}{format!("{:.4}", props.data.last().map(|p| p.macd_line).unwrap_or(0.0))}</div>
                <div>{"Signal: "}{format!("{:.4}", props.data.last().map(|p| p.signal_line).unwrap_or(0.0))}</div>
                <div>{"Histogram: "}{format!("{:.4}", props.data.last().map(|p| p.histogram).unwrap_or(0.0))}</div>
            </div>
        </div>
    }
}
