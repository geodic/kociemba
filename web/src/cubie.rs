use std::fmt;
use std::fmt::Display;

use yew::{html, Component, Html, classes};
use yew::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Color {
    White,
    Yellow,
    Red,
    Orange,
    Blue,
    Green,
    Gray,
}

impl Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", &self)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Face {
    Up,
    Right,
    Front,
    Down,
    Left,
    Back,
}

impl Display for Face {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", &self)
    }
}

#[derive(Clone, Copy)]
pub struct Cubie {
    pub id: u8,
    pub face: Face,
    pub color: Color,
}

impl Cubie {

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    pub fn new() ->Self {
        Self { id: 0,
            face: Face::Up,
            color: Color::Gray,
             }
    }
    
}

#[derive(Properties, Debug, PartialEq)]
pub struct CubieProps {
    pub id: u8,
    pub face: Face,
    pub color: Color,
    pub moving: bool,
}

impl Component for Cubie{
    type Message = ();
    type Properties = CubieProps;
    fn create(ctx: &Context<Self>) -> Self {
        Self {  id: ctx.props().id, face: Face::Up, color: ctx.props().color }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let id = format!("{}_{}", ctx.props().face, ctx.props().id);
        let color = match ctx.props().color {
            Color::White => "white",
            Color::Yellow => "yellow",
            Color::Red => "red",
            Color::Orange => "orange",
            Color::Blue => "blue",
            Color::Green => "green",
            Color::Gray => "gray",
        };
        let (center_str, center) = match ctx.props().id {
            4 => ("U", "center"),
            13 => ("R", "center"),
            22 => ("F", "center"),
            31 => ("D", "center"),
            40 => ("L", "center"),
            49 => ("B", "center"),
            _ => ("", ""),
        };
        let moving = match ctx.props().moving {
            true => "moving",
            false => "",
        };
        html!(<div class={classes!("game-cubie", color, center, moving)} id={id.clone()}>{center_str}</div>)
    }
}