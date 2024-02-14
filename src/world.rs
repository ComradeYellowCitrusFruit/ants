/*  
*   SPDX-License-Identifier: GPL-3.0-only
*   A very dumb little project simulating ants and complex behavior
*   Copyright (C) 2024 Teresa Maria Rivera
*/

use crate::ant::{Ant, Pheromones};

use super::shape::Shape;
use std::{any::TypeId, cell::RefCell, collections::{BinaryHeap, VecDeque}, rc::Rc};
use glm::{distance, vec2, Vec2};

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
            let mut phers = self.things.iter().filter(|o| (*o.borrow()).type_id() == TypeId::of::<Pheromones>());
            if phers.any(|o| o.borrow().contains_point(point)) {
                let val = phers.filter_map(|o| {
                    if o.borrow().contains_point(point) {
                        let tmp = o.borrow();
                        let pher = tmp.as_any().downcast_ref::<Pheromones>().unwrap();
                        Some((pher.strength/distance(pher.pos, src)) + 2.0)
                    } else {
                        None
                    }
                }).fold(2f32, |a, b| a + b);

                Some((point, val))
            } else {
                Some((point, 1.0f32))
            }
        }
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
                let mut path = VecDeque::from([cur.0]);
                let mut this = cur;

                while this.0 != src.pos {
                    this = astar[this.6].clone();
                    path.push_front(this.0);
                }

                return Some(path.make_contiguous().to_vec());
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
}