use yew::prelude::*;
mod styles;

// -- BoxData: represents each child box --
#[derive(Clone, PartialEq)]
struct BoxData {
    label: &'static str,
    title: &'static str,
    value: &'static str,
    bar_width: u8,
    color_cls: &'static str,
    wide: bool,
}

// -- All 9 boxes --
const BOXES: [BoxData; 9] = [
    BoxData {
        label: "Box 01",
        title: "CPU Usage",
        value: "74%",
        bar_width: 74,
        color_cls: "c1",
        wide: false,
    },
    BoxData {
        label: "Box 02",
        title: "Memory",
        value: "3.2 GB",
        bar_width: 55,
        color_cls: "c2",
        wide: false,
    },
    BoxData {
        label: "Box 03",
        title: "Network",
        value: "↑ 88ms",
        bar_width: 30,
        color_cls: "c3",
        wide: false,
    },
    BoxData {
        label: "Box 04 — Wide",
        title: "Disk I/O Activity",
        value: "1.4 TB",
        bar_width: 80,
        color_cls: "c4",
        wide: true,
    },
    BoxData {
        label: "Box 05",
        title: "Threads",
        value: "128",
        bar_width: 60,
        color_cls: "c5",
        wide: false,
    },
    BoxData {
        label: "Box 06",
        title: "Errors",
        value: "0",
        bar_width: 0,
        color_cls: "c6",
        wide: false,
    },
    BoxData {
        label: "Box 07",
        title: "Requests",
        value: "4.2k",
        bar_width: 90,
        color_cls: "c7",
        wide: false,
    },
    BoxData {
        label: "Box 08 — Wide",
        title: "Active Sessions",
        value: "217 live",
        bar_width: 65,
        color_cls: "c8",
        wide: true,
    },
    BoxData {
        label: "Box 09",
        title: "Uptime",
        value: "99.9%",
        bar_width: 99,
        color_cls: "c9",
        wide: false,
    },
];

#[derive(Clone)]
enum Msg {
    Rotate,
}

struct MasterBox {
    step: i32,
    angle: i32,
}

impl Component for MasterBox {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self { step: 0, angle: 0 }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Rotate => {
                self.step += 1;
                self.angle = self.step * 90;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        let on_rotate = link.callback(|_| Msg::Rotate);

        let transform_style = format!("transform: rotate({}deg)", self.angle);

        let display_angle = self.angle % 360;

        html! {
            <>
                <style>{ styles::CSS }</style>
                <div class="controls">
                    <button class="rotate-btn" onclick={on_rotate}>
                        { "⟳ Rotate" }
                    </button>
                    <div class="step-label">
                        { "Step: " }
                        <span>{ self.step }</span>
                        { " | Angle: "  }
                        <span>{ format!("{}°", display_angle) }</span>
                    </div>
                </div>

                <div class="scene">
                    <div class="master-box" id="masterBox" style={transform_style}>
                        { for BOXES.iter().map(|b| {
                            let cls = if b.wide {
                                format!("box {} wide", b.color_cls)
                            } else {
                                format!("box {}", b.color_cls)
                            };
                            let bar_w = format!("width:{}%", b.bar_width);

                            html! {
                                <div class={cls}>
                                    <div class="box-label">{ b.label }</div>
                                    <div class="box-title">{ b.title }</div>
                                    <div class="box-value">{ b.value }</div>
                                    <div class="bar-track">
                                        <div class="bar-fill" style={bar_w} />
                                    </div>
                                </div>
                            }
                        })}
                    </div>
                </div>
            </>
        }
    }
}

fn main() {
    yew::Renderer::<MasterBox>::new().render();
}
