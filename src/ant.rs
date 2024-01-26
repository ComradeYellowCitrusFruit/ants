/*  
*   SPDX-License-Identifier: GPL-3.0-only
*   A very dumb little project simulating ants and complex behavior
*   Copyright (C) 2024 Teresa Maria Rivera
*/

use glm::Vector2;
use std::vec::Vector;

pub enum Location {
    Home, 
    Dest,
    Here,
    PheromoneSrc,
    Pos(Vector2),
}

pub enum Source {
    Dist(Location),
    Loc(Location),
    Food,
    Number(f32),
    Memory(i32),
    PheromoneStrength,
}

pub enum Condition {
    GreaterThan(Source, Source),
    LessThan(Source, Source),
    Equal(Source, Source),
    Not(Condition),
}

pub enum Then {
    SetDest,
    EmitPheromone,
    Remember(Then),
    Forget(Then),
}

pub enum Decision {
    If(Condition, Then),
    IfHaveFood(Then),
    Always(Then),
}

pub enum Memory {
    Number(f32),
    Position(Vector2),
}

// An ant.
pub struct Ant {
    pos: Vector2, // aka center of a circle with r=2
    decisions: [Decision; 4],
    memory: Vector<Memory>,
    has_food: bool,
}

// A source of pheromones
pub struct Pheromones {
    pos: Vector2,
    strength: f32, // apparent strength is calculated as (strength)/dist(p,a)
}