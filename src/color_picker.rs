use crate::lpf::Lpf;
use crate::math::*;
use bevy::color::*;
use indexmap::IndexMap;

pub const PICKER_INNER_RADIUS: f32 = 100.0;
pub const PICKER_MIDDLE_RADIUS: f32 = 180.0;
pub const PICKER_OUTER_BAND_WIDTH: f32 = 80.0;

pub struct ColorPicker {
    is_open: bool,
    pos: Vec2,
    inner_animation: Lpf,
    outer_animation: Lpf,
    colors: IndexMap<usize, ColorSelection>,
    active_node: Option<usize>,
    clicked_node: Option<usize>,
    preview_color: Option<Srgba>,
    selected_color: Option<Srgba>,
}

pub struct SelectionNode {
    pub color: Srgba,
    pub pos: Vec2,
    pub radius: Lpf,
}

impl SelectionNode {
    fn new(color: Srgba) -> Self {
        Self {
            color,
            pos: Vec2::ZERO,
            radius: Lpf::new(0.0, 0.0, 0.3),
        }
    }
}

struct ColorSelection {
    node: SelectionNode,
    secondary: IndexMap<usize, SelectionNode>,
}

fn generate_colors() -> IndexMap<usize, ColorSelection> {
    let mut id = 0;

    let palette = [
        "001219", "005f73", "0a9396", "94d2bd", "e9d8a6", "ee9b00", "ca6702", "bb3e03", "ae2012",
        "9b2226",
    ];

    let primary_colors = palette.into_iter().map(|c| Srgba::hex(c).unwrap());

    primary_colors
        .map(|color| {
            let node = SelectionNode::new(color);
            let secondary = (0..=10)
                .map(|l| {
                    id += 1;
                    let l = 0.1 + 0.8 * l as f32 / 10.0;
                    let color = color.with_luminance(l);
                    let node = SelectionNode::new(color);
                    (id, node)
                })
                .collect();
            id += 1;
            (id, ColorSelection { node, secondary })
        })
        .collect()
}

impl ColorPicker {
    pub fn new() -> Self {
        Self {
            is_open: false,
            pos: Vec2::ZERO,
            inner_animation: Lpf::new(0.0, 0.0, 0.3),
            outer_animation: Lpf::new(0.0, 0.0, 0.3),
            colors: generate_colors(),
            active_node: None,
            clicked_node: None,
            preview_color: None,
            selected_color: None,
        }
    }

    pub fn open(&mut self, p: Vec2) {
        if self.is_open {
            return;
        }
        self.is_open = true;
        self.pos = p;
    }

    pub fn close(&mut self) {
        if self.preview_color.is_some() {
            self.selected_color = self.preview_color;
        }
        self.is_open = false;
        self.active_node = None;
    }

    pub fn step(&mut self) {
        self.inner_animation.target = self.is_open as u8 as f32;
        self.inner_animation.step();
        self.outer_animation.target = self.is_outer_open() as u8 as f32;
        self.outer_animation.step();

        let or = self.outer_band_width();
        let r_middle_1 = (self.inner_radius() + self.middle_radius()) / 2.0;
        let r_middle_2 = (self.outer_radius() + self.middle_radius()) / 2.0;
        let r_sample = self.inner_animation.actual
            * ((self.middle_radius() - self.inner_radius()) / 2.0 - 10.0);
        let nmax = self.colors.len();

        for (i, (id, c)) in self.colors.iter_mut().enumerate() {
            let r_extra = if self.clicked_node == Some(*id) {
                15.0
            } else if self.active_node == Some(*id) {
                5.0
            } else {
                0.0
            };

            let a = 2.0 * std::f32::consts::PI * i as f32 / nmax as f32;
            let u = Vec2::from_angle(a + std::f32::consts::PI / 2.0);
            c.node.pos = self.pos + r_middle_1 * u;
            c.node.radius.target = r_sample + r_extra;
            c.node.radius.step();

            let children_active = self.active_node == Some(*id)
                || c.secondary
                    .iter()
                    .any(|(id, _)| self.active_node == Some(*id));

            let r_child = if children_active {
                (or / 2.0 - 10.0).max(0.0)
            } else {
                0.0
            };

            let jmax = c.secondary.len();
            for (j, (id, n)) in c.secondary.iter_mut().enumerate() {
                let r_extra = if self.clicked_node == Some(*id) {
                    15.0
                } else if self.active_node == Some(*id) {
                    5.0
                } else {
                    0.0
                };

                let a = 2.0 * std::f32::consts::PI * j as f32 / jmax as f32;
                let u = Vec2::from_angle(a + std::f32::consts::PI / 2.0);
                n.pos = self.pos + r_middle_2 * u;
                n.radius.target = r_child + r_extra;

                n.radius.step();
            }
        }

        if let Some(id) = self.active_node {
            if let Some(n) = self.find_node(id) {
                self.preview_color = Some(n.color);
            }
        } else {
            self.preview_color = None;
        }
    }

    pub fn center(&self) -> Vec2 {
        self.pos
    }

    pub fn inner_radius(&self) -> f32 {
        self.inner_animation.actual * PICKER_INNER_RADIUS
    }

    pub fn middle_radius(&self) -> f32 {
        self.inner_animation.actual * PICKER_MIDDLE_RADIUS
    }

    pub fn outer_radius(&self) -> f32 {
        self.middle_radius() + self.outer_animation.actual * PICKER_OUTER_BAND_WIDTH
    }

    pub fn outer_band_width(&self) -> f32 {
        self.outer_animation.actual * PICKER_OUTER_BAND_WIDTH
    }

    pub fn alpha(&self) -> f32 {
        self.inner_animation.actual
    }

    pub fn is_outer_open(&self) -> bool {
        self.active_node.is_some()
    }

    fn nodes_with_id(&self) -> impl Iterator<Item = (&usize, &SelectionNode)> + use<'_> {
        self.colors
            .iter()
            .flat_map(|(id, c)| [(id, &c.node)].into_iter().chain(c.secondary.iter()))
    }

    fn selection_node_at(&self, p: Vec2) -> Option<usize> {
        for (id, node) in self.nodes_with_id() {
            let d = node.pos.distance(p);
            if d < node.radius.actual {
                return Some(*id);
            }
        }
        None
    }

    pub fn set_cursor_position(&mut self, pos: Option<Vec2>) {
        if let Some(p) = pos {
            let active_node = self.selection_node_at(p);
            if active_node.is_some() {
                self.active_node = active_node;
            }

            let r1 = self.inner_radius();
            if self.pos.distance(p) < r1 {
                self.active_node = None;
            }
        } else {
            self.active_node = None;
        }
    }

    pub fn on_left_click_down(&mut self) {
        if !self.is_open {
            return;
        }

        self.clicked_node = self.active_node;
    }

    pub fn on_left_click_up(&mut self) {
        if !self.is_open {
            return;
        }

        self.clicked_node = None;
    }

    fn find_node(&self, id: usize) -> Option<&SelectionNode> {
        self.nodes_with_id()
            .find(|(n, _)| **n == id)
            .map(|(_, n)| n)
    }

    pub fn preview_color(&self) -> Option<Srgba> {
        self.preview_color
    }

    pub fn selected_color(&self) -> Option<Srgba> {
        self.selected_color
    }

    pub fn samplers(&self) -> impl Iterator<Item = &SelectionNode> + use<'_> {
        self.nodes_with_id().map(|(_, n)| n)
    }
}
