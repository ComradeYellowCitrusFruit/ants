/*  
*   SPDX-License-Identifier: GPL-3.0-only
*   A very dumb little project simulating ants and complex behavior
*   Copyright (C) 2024 Teresa Maria Rivera
*/

use std::vec::Vec;
use glm::Vector2;

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
    Circle(Vector2<f32>, f32),
    Rect(Vector2<f32>, Vector2<f32>),
    Other,
}

pub trait Shape {
    fn collides<T: Shape>(&self, shape: &T) -> bool;
    fn contains_point<T: Into<Vector2<f32>>>(&self, p: T) -> bool;
    fn into_points(&self) -> Vec<Vector2<f32>>;
    fn get_center(&self) -> Vector2<f32>;
    fn get_shape_type(&self) -> ShapeType;
    fn into_basic_shape(&self) -> BasicShape;
}