/*  
*   SPDX-License-Identifier: GPL-3.0-only
*   A very dumb little project simulating ants and complex behavior
*   Copyright (C) 2024 Teresa Maria Rivera
*/

use glm::{vec2, Vec2};
use std::vec::Vec;

use crate::{shape::{self, BasicShape, Shape, ShapeType}, world::square_dist};

#[derive(Copy, Clone)]
pub enum Location {
    Home, 
    Dest,
    Here,
    PheromoneSrc,
    Pos(Vec2),
}

#[derive(Clone)]
pub enum Source {
    Dist(Location),
    Loc(Location),
    Food,
    Number(f32),
    Memory(i32),
    PheromoneStrength,
}

#[derive(Clone)]
pub enum Condition {
    GreaterThan(Source, Source),
    LessThan(Source, Source),
    Equal(Source, Source),
    Not(Box<Condition>),
}

#[derive(Clone)]
pub enum Then {
    SetDest,
    EmitPheromone,
    Remember(Box<Then>),
    Forget(Box<Then>),
}

#[derive(Clone)]
pub enum Decision {
    If(Condition, Then),
    IfHaveFood(Then),
    Always(Then),
}

#[derive(Clone)]
pub enum Memory {
    Number(f32),
    Position(Vec2),
}

// An ant.
#[derive(Clone)]
pub struct Ant {
    pub(crate) pos: Vec2, // aka center of a circle with r=2 (in a 250x250 grid)
    decisions: [Decision; 4],
    memory: Vec<Memory>,
    has_food: bool,
}

impl Shape for Ant {
    fn collides(&self, shape: &dyn Shape) -> bool {
        match shape.into_basic_shape() {
            BasicShape::Circle(c, r) => {
                let dist =   ((self.pos.x - c.x) * (self.pos.x - c.x)) 
                                + ((self.pos.y - c.y) * (self.pos.y - c.y)) 
                                - (r*r);
                let ulp = (dist.to_bits() as i32 - 4.0f32.to_bits() as i32).abs();
                if ulp > 16 && dist > 0.001 {
                    false
                } else {
                    true
                }
            }
            BasicShape::Rect(cl, wh) => {
                let points = vec![
                    cl, vec2(cl.x, cl.y - (wh.y/2.0)), vec2(cl.x, cl.y - wh.y), 
                    vec2(cl.x + (wh.x/2.0), cl.y - wh.y), vec2(cl.x + wh.x, cl.y - wh.y),
                    vec2(cl.x + wh.x, cl.y - (wh.y/2.0)), vec2(cl.x + wh.x, cl.y),
                ];

                for p in points {
                    let dist = ((self.pos.x - p.x)*(self.pos.x - p.x)) + ((self.pos.y - p.y)*(self.pos.y - p.y));
                    let ulp = (dist.to_bits() as i32 - 4.0f32.to_bits() as i32).abs();

                    if ulp > 16 {
                        if dist < 0.001 {
                            return true;
                        }
                    }
                }

                false
            },
            BasicShape::Other => {
                for p in shape.into_points() {
                    let dist = ((self.pos.x - p.x)*(self.pos.x - p.x)) + ((self.pos.y - p.y)*(self.pos.y - p.y));
                    let ulp = (dist.to_bits() as i32 - 4.0f32.to_bits() as i32).abs();

                    if ulp > 16 {
                        if dist < 0.001 {
                            return true;
                        }
                    }
                }

                shape.collides(self)
            }
        }
    }

    fn contains_point(&self, p: Vec2) -> bool {
        let dist = ((self.pos.x - p.x) * (self.pos.x - p.x)) + ((self.pos.y - p.y) * (self.pos.y - p.y));
        let ulp = (dist.to_bits() as i32 - 4.0f32.to_bits() as i32).abs();

        if ulp > 16 && dist > 0.001 {
            false
        } else {
            true
        }
    }

    fn into_points(&self) -> Vec<Vec2> {
        (0..128).map(|i| vec2(2.0 * (2.8125 * (i as f32)).to_radians().sin(), 2.0 * (2.8125 * (i as f32)).to_radians().cos())).collect()
    }

    fn get_center(&self) -> Vec2 {
        self.pos
    }

    fn get_shape_type(&self) -> ShapeType {
        ShapeType::Circle
    }

    fn into_basic_shape(&self) -> crate::shape::BasicShape {
        BasicShape::Circle(self.pos, 2.0f32)
    }
}

// A source of pheromones
#[derive(Clone)]
pub struct Pheromones {
    pub(crate) pos: Vec2,
    pub(crate) strength: f32, // apparent strength is calculated as (strength)/dist(p,a)
}

impl Shape for Pheromones {
    fn collides(&self, shape: &dyn Shape) -> bool {
        match shape.into_basic_shape() {
            BasicShape::Circle(c, r) => {
                let dist = square_dist(self.pos, c) - r.powi(2);
                let ulp = (dist.to_bits() as i32 - (self.strength.powi(2)).to_bits() as i32).abs();
                if ulp > 16 && dist > 0.001 {
                    false
                } else {
                    true
                }
            }
            BasicShape::Rect(cl, wh) => {
                let points = vec![
                    cl, vec2(cl.x, cl.y - (wh.y/2.0)), vec2(cl.x, cl.y - wh.y), 
                    vec2(cl.x + (wh.x/2.0), cl.y - wh.y), vec2(cl.x + wh.x, cl.y - wh.y),
                    vec2(cl.x + wh.x, cl.y - (wh.y/2.0)), vec2(cl.x + wh.x, cl.y),
                ];

                for p in points {
                    let dist = square_dist(self.pos, p);
                    let ulp = (dist.to_bits() as i32 - (self.strength.powi(2)).to_bits() as i32).abs();

                    if ulp > 16 {
                        if dist < 0.001 {
                            return true;
                        }
                    }
                }

                false
            },
            BasicShape::Other => {
                for p in shape.into_points() {
                    let dist = square_dist(self.pos, p);
                    let ulp = (dist.to_bits() as i32 - (self.strength.powi(2)).to_bits() as i32).abs();

                    if ulp > 16 {
                        if dist < 0.001 {
                            return true;
                        }
                    }
                }

                shape.collides(self)
            }
        }
    }

    fn contains_point(&self, p: Vec2) -> bool {
        let dist = square_dist(self.pos, p);
        let ulp = (dist.to_bits() as i32 - (self.strength.powi(2)).to_bits() as i32).abs();

        if ulp > 16 && dist > 0.001 {
            false
        } else {
            true
        }
    }

    fn into_points(&self) -> Vec<Vec2> {
        (0..128).map(|i| vec2(self.strength * (2.8125 * (i as f32)).sin(), self.strength * (2.8125 * (i as f32)).cos())).collect()
    }

    fn get_center(&self) -> Vec2 {
        self.pos
    }

    fn get_shape_type(&self) -> ShapeType {
        ShapeType::Circle
    }

    fn into_basic_shape(&self) -> crate::shape::BasicShape {
        BasicShape::Circle(self.pos, self.strength)
    }
}