use crate::cubie::Face;
use cubie::{Color, Cubie};
use ehttp;
use gloo::console::log;
use gloo::timers::callback::Interval;
use kociemba::cubie::CubieCube;
use kociemba::facelet::{self, FaceCube};
use kociemba::moves::Move;
use kociemba::scramble::scramble_from_str;
use kociemba::solver::SoutionResult;
use std::collections::VecDeque;
use std::fmt::format;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use web_sys::HtmlTextAreaElement as InputElement;
use yew::events::{FocusEvent, KeyboardEvent};
use yew::{html, Component, Context, Html, TargetCast};

mod cubie;

pub enum Msg {
    Random,
    Clean,
    Solve,
    Step,
    Stop,
    SetColor(Color),
    Tick,
    SetFacelet(String),
}

pub struct App {
    active: bool,
    cubies: Vec<Cubie>,
    facelet: String,
    _interval: Interval,
    solution: Arc<Mutex<SoutionResult>>,
    command_queue: VecDeque<Move>,
    movings: Vec<u8>,
    movingface: Option<Move>,
}

impl App {
    pub fn solve(&mut self) {
        let host = "127.0.0.1";
        let port = 32125;
        let facelet = self.facelet.clone();
        let url = format!("http://{}:{}/solve/{}", host, port, facelet);
        let c_solution = Arc::clone(&self.solution);
        let request = ehttp::Request {
            headers: ehttp::Headers::new(&[
                ("Accept", "*/*"),
                // ("Content-Type", "application/json;charset=utf-8"),
                // ("Access-Control-Allow-Headers", "Accept"),
            ]),
            ..ehttp::Request::get(url)
        };
        ehttp::fetch(request, move |result: ehttp::Result<ehttp::Response>| {
            let response = result.unwrap().bytes;
            let _solution: SoutionResult = serde_json::from_slice(&response).unwrap();
            let mut slock = c_solution.lock().unwrap();
            *slock = _solution.clone();
            log!(format!("Solution in closure: {:?}", *slock));
        });
    }

    fn step(&mut self) {
        let solution = Arc::clone(&self.solution);
        let mut solution = solution.lock().unwrap();
        if (*solution).solution.len() > 0 {
            self.command_queue = VecDeque::from((*solution).solution.clone());
            (*solution).solution = Vec::new();
        }
        let fc = FaceCube::try_from(self.facelet.as_str()).unwrap();
        let cc = CubieCube::try_from(&fc).unwrap();
        match self.command_queue.pop_front() {
            Some(step) => {
                self.movingface = Some(step);
                log!(format!("Step: {}", step));
                self.movings = match step {
                    Move::R | Move::R2 | Move::R3 => {
                        vec![
                            2, 5, 8, 20, 23, 26, 29, 32, 35, 45, 48, 51, 9, 10, 11, 12, 13, 14, 15,
                            16, 17,
                        ]
                    }
                    Move::U | Move::U2 | Move::U3 => {
                        vec![
                            0, 1, 2, 3, 4, 5, 6, 7, 8, 36, 37, 38, 18, 19, 20, 9, 10, 11, 45, 46,
                            47,
                        ]
                    }
                    Move::L | Move::L2 | Move::L3 => {
                        vec![
                            36, 37, 38, 39, 40, 41, 42, 43, 44, 0, 3, 6, 18, 21, 24, 27, 30, 33,
                            47, 50, 53,
                        ]
                    }
                    Move::F | Move::F2 | Move::F3 => {
                        vec![
                            18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 6, 7, 8, 38, 41, 44, 9,
                            12, 15,
                        ]
                    }
                    Move::D | Move::D2 | Move::D3 => {
                        vec![
                            27, 28, 29, 30, 31, 32, 33, 34, 35, 42, 43, 44, 24, 25, 26, 15, 16, 17,
                            51, 52, 53,
                        ]
                    }
                    Move::B | Move::B2 | Move::B3 => {
                        vec![
                            45, 46, 47, 48, 49, 50, 51, 52, 53, 0, 1, 2, 36, 39, 42, 11, 14, 17,
                            33, 34, 35,
                        ]
                    }
                };
                let cc = cc.apply_move(step);
                let fc = FaceCube::try_from(&cc).unwrap();
                self.facelet = fc.to_string();
            }
            None => {
                self.active = false;
                self.movings = Vec::new();
                self.movingface = None;
            }
        };
    }
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let callback = ctx.link().callback(|_| Msg::Tick);
        let interval = Interval::new(1000, move || callback.emit(()));

        Self {
            active: false,
            cubies: vec![Cubie::new(); 54],
            facelet: "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB".to_string(),
            _interval: interval,
            solution: Arc::new(Mutex::new(SoutionResult::default())),
            command_queue: VecDeque::new(),
            movings: Vec::new(),
            movingface: None,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Random => {
                let mut cc = CubieCube::default();
                cc.randomize();
                let fc = FaceCube::try_from(&cc).unwrap();
                self.facelet = format!("{}", fc);
                // self.facelet = "URUUUFLLDFURBRRDBDUDRLFDFFFUDLRDUBFBLLFULBDDRBFBBBRRLL".to_string();
                self.solve();
                self.active = false;
                true
            }
            Msg::Solve => {
                self.active = true;
                self.step();
                true
            }
            Msg::Step => {
                self.active = false;
                self.step();
                true
            }
            Msg::Clean => {
                self.active = false;
                self.facelet = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB".to_string();
                self.solution = Arc::new(Mutex::new(SoutionResult::default()));
                self.command_queue = VecDeque::new();
                self.movings = Vec::new();
                self.movingface = None;
                true
            }
            Msg::Stop => {
                self.active = false;
                log::info!("Stop");
                false
            }
            Msg::SetColor(color) => {
                let mut cellule = self.cubies[0];
                cellule.set_color(color);
                true
            }
            Msg::Tick => {
                if self.active {
                    self.step();
                    true
                } else {
                    false
                }
            }
            Msg::SetFacelet(facelet) => {
                self.facelet = facelet;
                self.solve();
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let facelet = self.facelet.as_bytes();

        let mut rows = Vec::new();

        let face = Face::Up;
        let mut cubies_up = Vec::new();
        for i in 0..3 {
            for j in 0..3 {
                let id = i * 3 + j;
                let is_moving = self.movings.contains(&id);
                let color = f2c(facelet[id as usize].into());
                let c = html! {<Cubie id={id as u8} color={color} face={face} moving={is_moving}/>};
                cubies_up.push(c);
            }
        }
        let next_moving = match self.command_queue.len() > 0 {
            true => self.command_queue[0].to_string(),
            false => "".to_string(),
        };
        let current_moving = match self.movingface {
            Some(moving) => moving.to_string(),
            None => "".to_string(),
        };
        let row = html! {
            <div class="up_layer">
                <div></div>
                <div class="face" id="up">{for cubies_up}</div>
                <div class="info">
                    <h2>{format!("Moving Queue: {:?}", self.command_queue)}</h2>
                    <h2>{format!("Current Moving: {}", current_moving)}</h2>
                    <h2>{format!("Next Moving: {}", next_moving)}</h2>
                </div>
            </div>
        };
        rows.push(row);

        let face_left = Face::Left;
        let mut cubies_left = Vec::new();
        for i in 0..3 {
            for j in 0..3 {
                let id = i * 3 + j + 36;
                let is_moving = self.movings.contains(&id);
                let color = f2c(facelet[id as usize].into());
                let c = html! {<Cubie id={id as u8} color={color} face={face_left} moving={is_moving}/>};
                cubies_left.push(c);
            }
        }
        let face_front = Face::Front;
        let mut cubies_front = Vec::new();
        for i in 0..3 {
            for j in 0..3 {
                let id = i * 3 + j + 18;
                let is_moving = self.movings.contains(&id);
                let color = f2c(facelet[id as usize].into());
                let c = html! {<Cubie id={id as u8} color={color} face={face_front} moving={is_moving}/>};
                cubies_front.push(c);
            }
        }
        let face_right = Face::Right;
        let mut cubies_right = Vec::new();
        for i in 0..3 {
            for j in 0..3 {
                let id = i * 3 + j + 9;
                let is_moving = self.movings.contains(&id);
                let color = f2c(facelet[id as usize].into());
                let c = html! {<Cubie id={id as u8} color={color} face={face_right} moving={is_moving}/>};
                cubies_right.push(c);
            }
        }
        let face_back = Face::Back;
        let mut cubies_back = Vec::new();
        for i in 0..3 {
            for j in 0..3 {
                let id = i * 3 + j + 45;
                let is_moving = self.movings.contains(&id);
                let color = f2c(facelet[id as usize].into());
                let c = html! {<Cubie id={id as u8} color={color} face={face_back} moving={is_moving}/>};
                cubies_back.push(c);
            }
        }
        let row = html! {
            <div class="middle_layer">
            <div class="face" id="left">{for cubies_left}</div>
            <div class="face" id="front">{for cubies_front}</div>
            <div class="face" id="right">{for cubies_right}</div>
            <div class="face" id="backcs">{for cubies_back}</div>
            </div>
        };
        rows.push(row);

        let face_down = Face::Down;
        let mut cubies_down = Vec::new();
        for i in 0..3 {
            for j in 0..3 {
                let id = i * 3 + j + 27;
                let is_moving = self.movings.contains(&id);
                let color = f2c(facelet[id as usize].into());
                let c = html! {<Cubie id={id as u8} color={color} face={face_down} moving={is_moving}/>};
                cubies_down.push(c);
            }
        }
        let onkeypress = ctx.link().callback(|e: KeyboardEvent| {
            if e.key() == "Enter" {
                let input: InputElement = e.target_unchecked_into();
                let value = input.value();
                
                match scramble_from_str(&value) {
                    Ok(moves) => {
                        let cc = CubieCube::default();
                        let cc = cc.apply_moves(&moves);
                        let fc = FaceCube::try_from(&cc).unwrap();
                        Msg::SetFacelet(fc.to_string())
                    },
                    Err(_e) => Msg::Clean,
                }
            } else {
                Msg::Clean
            }
        });
        let row = html! {
            <div class="down_layer">
            <div></div>
            <div class="face" id="down">{for cubies_down}</div>
            <div class="control">
                <p>{"Scramble the cube, input a scramble string, press ENTER"}</p>
                <label for="scramble" />
                <textarea class="scramble" id="scramble" textLength="60" placeholder="D' L F2 B U' R' B2 U' B F' L2 U' R2 L F2 L2 D2 U2 R' F'" {onkeypress}/>
            </div>
            </div>
        };
        rows.push(row);

        html! {
            <div>
                <div class="game-container">
                    <header class="app-header">
                    <div>
                        <img alt="The app logo" src="favicon.ico" class="app-logo"/>
                        </div>
                        <div>
                        <h1 class="app-title">{ "Rubik's Cube's Explorer" }</h1></div>
                    </header>
                    <div class="game-area">
                        <div class="game-facelet">
                            { for rows }
                        </div>
                        <div class="game-buttons">
                            <button class="game-button" onclick={ctx.link().callback(|_| Msg::Random)}>{ "Random" }</button>
                            <button class="game-button" onclick={ctx.link().callback(|_| Msg::Clean)}>{ "Clean" }</button>
                            <button class="game-button" onclick={ctx.link().callback(|_| Msg::Solve)}>{ "Solve" }</button>
                            <button class="game-button" onclick={ctx.link().callback(|_| Msg::Stop)}>{ "Stop" }</button>
                            <button class="game-button" onclick={ctx.link().callback(|_| Msg::Step)}>{ "Step" }</button>
                        </div>
                    </div>
                </div>
                <footer class="app-footer">
                    <strong class="footer-text">
                      { "Copyright @ Adun Gaos" }
                    </strong>
                </footer>
            </div>
        }
    }
}

pub fn f2c(f: char) -> Color {
    match f {
        'U' => Color::Yellow,
        'L' => Color::Blue,
        'F' => Color::Red,
        'R' => Color::Green,
        'B' => Color::Orange,
        _ => Color::White,
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    log::trace!("Initializing yew...");
    yew::Renderer::<App>::new().render();
}
