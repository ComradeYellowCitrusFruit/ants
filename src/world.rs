/*  
*   SPDX-License-Identifier: GPL-3.0-only
*   A very dumb little project simulating ants and complex behavior
*   Copyright (C) 2024 Teresa Maria Rivera
*/

use crate::ant::{Ant, Condition, Decision, Location, Memory, Pheromones, Source, Then};

use super::shape::Shape;
use std::{any::TypeId, cell::RefCell, collections::{BinaryHeap, VecDeque}, rc::Rc};
use glm::{distance, greaterThan, lessThan, vec2, Vec2};

#[derive(Clone)]
pub struct Environment {
    // all of these objects should share the same memory space across references
    things:    Vec<Rc<RefCell<dyn Shape>>>,
    colliders: Vec<Rc<RefCell<dyn Shape>>>,
    renderers: Vec<Rc<RefCell<dyn Shape>>>,
    ants:      Vec<Rc<RefCell<dyn Shape>>>, // these guys are special
}

#[derive(Clone)]
struct AStar(Vec2, f32, f32, Vec<usize>, bool, usize, usize);

impl PartialEq for AStar {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1 && self.2 == other.2 && self.3 == other.3
    }
}

impl Eq for AStar {}

impl PartialOrd for AStar {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        other.2.partial_cmp(&other.2).and_then(|a| Some(a.then_with(|| self.2.partial_cmp(&other.2).unwrap())))
    }
}

impl Ord for AStar {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

pub fn square_dist(a: Vec2, b: Vec2) -> f32 {
    (a.x - b.x).powi(2) + (a.y - b.y).powi(2)
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            things: Vec::new(),
            colliders: Vec::new(),
            renderers: Vec::new(),
            ants: Vec::new(),
        }
    }

    pub fn add<T: Shape + Clone + 'static>(&mut self, obj: T, flags: i32) {
        let tmp = Rc::new(RefCell::new(obj.clone()));
        self.things.push(Rc::clone(&tmp) as Rc<RefCell<dyn Shape>>);
        if (flags >> 0) & 1 == 1 {
            self.colliders.push(Rc::clone(&tmp) as Rc<RefCell<dyn Shape>>);
        }
        if (flags >> 1) & 1 == 1 {
            self.renderers.push(Rc::clone(&tmp) as Rc<RefCell<dyn Shape>>);
        }
        if (flags >> 2) & 1 == 1 {
            self.ants.push(Rc::clone(&tmp) as Rc<RefCell<dyn Shape>>);
        }
    }

    pub fn rm(&mut self, obj: Rc<RefCell<dyn Shape>>) {
        self.things = self.things.clone().into_iter().filter(|t| Rc::ptr_eq(t, &obj)).collect::<Vec<_>>();
        self.colliders = self.colliders.clone().into_iter().filter(|t| Rc::ptr_eq(t, &obj)).collect::<Vec<_>>();
        self.renderers = self.renderers.clone().into_iter().filter(|t| Rc::ptr_eq(t, &obj)).collect::<Vec<_>>();
        self.ants = self.ants.clone().into_iter().filter(|t| Rc::ptr_eq(t, &obj)).collect::<Vec<_>>();
    }

    fn process_point<'a>(&'a self, src: Vec2, point: Vec2, walker: &mut Ant) -> Option<(Vec2, f32)> {
        walker.pos = point;
        if self.colliders.iter().all(|o| walker.collides(&*o.borrow())) ||
           self.colliders.iter().all(|o| o.borrow().contains_point(point)) 
        {
            None
        } else {
            if self.things.iter().any(|o| o.borrow().type_id() == TypeId::of::<Pheromones>()) {
                Some((point, self.pheromone_strength_at_pos(point) + 2.0))
            } else {
                Some((point, 1.0f32))
            }
        }
    }

    pub fn pheromone_strength_at_pos(&self, pos: Vec2) -> f32 {
        self.things.clone().into_iter().filter_map(|r| {
            if  r.borrow().type_id() != TypeId::of::<Pheromones>() ||
                !r.borrow().contains_point(pos) 
            {
                return None;
            }

            let p = (*r.borrow()).downcast_ref::<Pheromones>().unwrap();
            Some(p.strength/distance(p.pos, pos))
        }).fold(0f32, |acc, s| acc + s)
    }

    pub fn chart_path(&self, src: &Ant, dest: Vec2) -> Option<Vec<Vec2>> {
        // this function is highly inefficient with memory, partially due to Rust's rules
        // TODO: improve memory efficiency

        let mut walker = src.clone(); // Used to check for collisions
        let mut grid: Vec<(Vec2,f32)> = vec![(src.pos, 0.0)];

        // first, generate a grid (well, list of points and scores), 
        // containing all points along a series of concentric circles
        // with the radius of each circle increasing by 1, until being within 
        // 2 units of the destination
        let mut r = 1;
        loop {
            let p: Vec<_> = (0..((360*r)/60)).map(|j| vec2(
                ((r as f32) * (((r as f32)/60.0) * (j as f32)).to_radians().sin()) + src.pos.x, 
                ((r as f32) * (((r as f32)/60.0) * (j as f32)).to_radians().cos()) + src.pos.y
            )).collect();

            grid.append(&mut p.into_iter().filter_map(|c| self.process_point(src.pos, c, &mut walker)).collect::<Vec<_>>());
            if distance(dest, src.pos) - (r as f32) <= 10.0 {
                break;
            }
            
            r += 1;
        }
        
        // trust dest
        grid.push((dest, f32::MAX/20.0));

        // next, we run A* (instead of just dist we do dist/val)

        // these are the points processed for A*
        let mut astar = grid.clone().into_iter().filter_map(|a| {
            let mut r = AStar(vec2(0.0,0.0), 0.0f32, 0.0f32, Vec::new(), false, 0, 0);
            r.0 = a.0;

            if a.1.is_nan() || a.1.is_infinite() {
                return None;
            }

            r.3.append(&mut grid.clone().into_iter().enumerate().filter_map(|o| {
                if square_dist(o.1.0, a.0) <= 2.25f32 {
                    Some(o.0)
                } else if o.1.0 == dest && square_dist(a.0, o.1.0) <= 100f32 {
                    Some(o.0)
                } else {
                    None
                }
            }).collect());
            
            if square_dist(a.0, dest) <= 100f32 {
                r.3.push(grid.len() - 1);
            }

            // calculate h
            r.1 = ((a.0.x - dest.x).abs() + (a.0.y - dest.y).abs())/a.1;

            if r.0 == src.pos {
                r.2 = r.1;
            } else {
                r.2 = f32::INFINITY;
            }

            if r.1.is_nan() || r.1.is_infinite() {
                None
            } else {
                Some(r)
            }
        }).enumerate().map(|a| { let mut r = a.1; r.5 = a.0; r}).collect::<Vec<_>>();

        // actually do A*
        let mut open = BinaryHeap::from([astar[0].clone()]);
        while open.is_empty() == false {
            let cur = open.pop().unwrap();

            if cur.0 == dest {
                let mut path = Vec::new();
                let mut this = cur;
                
                path.push(this.0);
                while this.0 != src.pos {
                    this = astar[this.6].clone();
                    path.push(this.0);
                }

                return Some(path);
            }

            for i in cur.3.clone().into_iter() {
                let n = &mut astar[i];
                let o = n.clone();
                let g = cur.1 + distance(cur.0, n.0);

                if g < (n.2 - n.1) {
                    n.2 = g + n.1;
                    n.6 = cur.5;

                    if open.iter().any(|a| *a == o) {
                        open.retain(|a| *a != o);
                        open.push(n.clone());
                    } else if open.iter().any(|a| a != n) {
                        open.push(n.clone());
                    }
                }
            }
        }

        None
    }

    fn get_location(&self, src: &Ant, loc: Location) -> Vec2 {
        match loc {
            Location::Here => src.pos,
            Location::Home | Location::Dest => todo!(),
            Location::Pos(p) => p,
            Location::PheromoneSrc => {
                let a = self.things.clone().into_iter().filter_map(|a| {
                    if a.borrow().type_id() != TypeId::of::<Pheromones>() {
                        return None;
                    }

                    let p = (*a.borrow()).downcast_ref::<Pheromones>().unwrap();
                    Some(p.pos)
                });
                a.fold(vec2(f32::MAX, f32::MAX), |acc, p| {
                    if square_dist(acc, src.pos) < square_dist(p, src.pos) {
                        p
                    } else {
                        acc
                    }
                })                
            },
        }
    }

    fn evaluate_src(&self, src: &Ant, source: Source) -> Memory {
        match source {
            Source::Number(n) => Memory::Number(n),
            Source::Dist(a) => {
                let target = match a {
                    Location::Here => return Memory::Number(0.0),
                    _ => self.get_location(src, a),
                };

                Memory::Number(distance(target, src.pos))
            },
            Source::Memory(i) => src.memory[src.memory.len() - (i as usize + 1)],
            Source::PheromoneStrength => Memory::Number(self.pheromone_strength_at_pos(src.pos)),
            Source::Food => todo!(),
            Source::Loc(l) => Memory::Position(self.get_location(src, l)),
        }
    }

    fn evaluate_cond(&self, src: &Ant, cond: Condition) -> bool {
        match cond {
            Condition::Equal(a, b) => {
                let aa = self.evaluate_src(src, a);
                let bb = self.evaluate_src(src, b);
                aa == bb
            },
            Condition::Not(cond) => !self.evaluate_cond(src, *cond),
            Condition::LessThan(a, b) => {
                match self.evaluate_src(src, a) {
                    Memory::Number(aa) => {
                        match self.evaluate_src(src, b) {
                            Memory::Number(bb) => aa < bb,
                            _ => false,
                        }
                    },
                    Memory::Position(aa) => {
                        match self.evaluate_src(src, b) {
                            Memory::Position(bb) => lessThan(aa, bb).x && lessThan(aa, bb).y,
                            _ => false,
                        }
                    }
                }
            },
            Condition::GreaterThan(a, b) => {
                match self.evaluate_src(src, a) {
                    Memory::Number(aa) => {
                        match self.evaluate_src(src, b) {
                            Memory::Number(bb) => aa > bb,
                            _ => false,
                        }
                    },
                    Memory::Position(aa) => {
                        match self.evaluate_src(src, b) {
                            Memory::Position(bb) => greaterThan(aa, bb).x && greaterThan(aa, bb).y,
                            _ => false,
                        }
                    }
                }
            }
        }
    }

    fn make_decision<F: FnMut(Then)>(&self, src: &Ant, d: Decision, mut f: F) {
        match d {
            Decision::Always(t) => f(t.clone()),
            Decision::If(c, t) => {
                if self.evaluate_cond(src, c) {
                    f(t.clone())
                }
            }
            Decision::IfHaveFood(t) => {
                if src.has_food {
                    f(t.clone())
                }
            }
        }
    }

    pub fn step(&mut self) {
        // first, process each pheromone
        self.things.iter_mut().for_each(|t| {
            if (*t.borrow()).type_id() == TypeId::of::<Pheromones>() {
                t.borrow_mut().downcast_mut::<Pheromones>().unwrap().strength -= 0.1;
            }
        });

        // now, actual ant behaior.
        for a in &self.ants {
            let mut tmp = a.borrow_mut();
            let ant = (*tmp).downcast_mut::<Ant>().unwrap();

            // their brains, ants have simple brains
            for d in &ant.decisions {
                self.make_decision(&ant.clone(), d.clone(), |mut t| {
                    loop {
                        match t {
                            Then::Forget(b) => { 
                                let _ = ant.memory.pop_front();
                                t = *b; 
                                continue; 
                            },
                            _ => todo!(),
                        }
                    }
                });
            }
        }
    }
}
