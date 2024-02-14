/*  
*   SPDX-License-Identifier: GPL-3.0-only
*   A very dumb little project simulating ants and complex behavior
*   Copyright (C) 2024 Teresa Maria Rivera
*/

use std::vec::Vec;
use glm::Vec2;
use downcast_rs::{impl_downcast, Downcast};

#[derive(Clone, Copy)]
pub enum ShapeType {
    Circle,
    Rect,
    RegularPolygon,
    Star,
    Other
}

#[derive(Clone, Copy)]
pub enum BasicShape {
    Circle(Vec2, f32),
    Rect(Vec2, Vec2),
    Other,
}

pub trait Shape: Downcast {
    fn collides(&self, shape: &dyn Shape) -> bool;
    fn contains_point(&self, p: Vec2) -> bool;
    fn into_points(&self) -> Vec<Vec2>;
    fn get_center(&self) -> Vec2;
    fn get_shape_type(&self) -> ShapeType;
    fn into_basic_shape(&self) -> BasicShape;
}

impl_downcast!(Shape);