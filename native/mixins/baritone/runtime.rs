//! Baritone-style goals for Horizon. Java Baritone does not run here.
//! This walks the local player across loaded chunks and mines one block at a time.

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::sync::Mutex;

pub const ID: &str = "baritone";
pub const NAME: &str = "Baritone";
pub const VERSION: &str = "0.1.0";
pub const DESCRIPTION: &str =
    "Ходит к блоку и копает. /b goto, /b mine, /b stop. Только загруженные чанки.";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Goal {
    Idle,
    Goto([i32; 3]),
    Mine(String),
}

#[derive(Clone, Debug)]
struct Move {
    forward: f32,
    jump: bool,
    on: bool,
}

impl Default for Move {
    fn default() -> Self {
        Self {
            forward: 0.0,
            jump: false,
            on: false,
        }
    }
}

struct Bot {
    goal: Goal,
    path: Vec<[i32; 3]>,
    status: String,
    brk: Option<[i32; 3]>,
    stuck: [f64; 3],
    stuck_ticks: u32,
    replan: u32,
}

impl Bot {
    fn new() -> Self {
        Self {
            goal: Goal::Idle,
            path: Vec::new(),
            status: "стоп".into(),
            brk: None,
            stuck: [0.0; 3],
            stuck_ticks: 0,
            replan: 0,
        }
    }
}

static BOT: Mutex<Bot> = Mutex::new(Bot {
    goal: Goal::Idle,
    path: Vec::new(),
    status: String::new(),
    brk: None,
    stuck: [0.0; 3],
    stuck_ticks: 0,
    replan: 0,
});

fn with_bot<T>(f: impl FnOnce(&mut Bot) -> T) -> T {
    let mut bot = BOT.lock().unwrap_or_else(|p| p.into_inner());
    if bot.status.is_empty() {
        *bot = Bot::new();
    }
    f(&mut bot)
}

pub fn command_usages() -> Vec<String> {
    ["b", "b help", "b stop", "b goto", "b mine", "baritone"]
        .into_iter()
        .map(str::to_owned)
        .collect()
}

/// Chat command without the slash. Empty when it is not ours.
pub fn handle(line: &str) -> Option<Vec<String>> {
    let line = line.trim();
    let rest = line
        .strip_prefix("baritone")
        .or_else(|| line.strip_prefix('b'))?;
    if !rest.is_empty() && !rest.starts_with(|c: char| c.is_whitespace()) {
        return None;
    }
    let args: Vec<&str> = rest.split_whitespace().collect();
    Some(match args.first().copied() {
        None | Some("help") => vec![
            "Baritone: /b goto <x> <y> <z>".into(),
            "/b mine <блок>   /b stop".into(),
        ],
        Some("stop") | Some("cancel") => {
            with_bot(|bot| {
                *bot = Bot::new();
            });
            vec!["Baritone остановлен.".into()]
        }
        Some("goto") | Some("goal") => {
            if args.len() < 4 {
                return Some(vec!["Нужны три числа: /b goto <x> <y> <z>".into()]);
            }
            let parsed = parse3(&args[1], &args[2], &args[3]);
            let Some(at) = parsed else {
                return Some(vec!["Координаты должны быть целыми.".into()]);
            };
            with_bot(|bot| {
                bot.goal = Goal::Goto(at);
                bot.path.clear();
                bot.brk = None;
                bot.replan = 0;
                bot.status = format!("иду к {} {} {}", at[0], at[1], at[2]);
            });
            vec![format!("Цель: {} {} {}.", at[0], at[1], at[2])]
        }
        Some("mine") => {
            let Some(name) = args.get(1) else {
                return Some(vec!["Нужен id блока: /b mine diamond_ore".into()]);
            };
            let id = block_id(name);
            with_bot(|bot| {
                bot.goal = Goal::Mine(id.clone());
                bot.path.clear();
                bot.brk = None;
                bot.replan = 0;
                bot.status = format!("ищу {id}");
            });
            vec![format!("Ищу {id} в загруженных чанках.")]
        }
        Some(other) => vec![format!("Неизвестная команда «{other}». /b help")],
    })
}

fn parse3(x: &str, y: &str, z: &str) -> Option<[i32; 3]> {
    Some([x.parse().ok()?, y.parse().ok()?, z.parse().ok()?])
}

fn block_id(name: &str) -> String {
    if name.contains(':') {
        name.to_owned()
    } else {
        format!("minecraft:{name}")
    }
}

pub fn hud_lines() -> Vec<String> {
    with_bot(|bot| {
        if bot.goal == Goal::Idle {
            vec!["Baritone".into(), "/b help".into()]
        } else {
            vec!["Baritone".into(), bot.status.clone()]
        }
    })
}

pub fn take_move() -> (f32, bool, bool) {
    let m = MOVE.lock().unwrap_or_else(|p| p.into_inner());
    (m.forward, m.jump, m.on)
}

static MOVE: Mutex<Move> = Mutex::new(Move {
    forward: 0.0,
    jump: false,
    on: false,
});

pub fn take_break() -> Option<[i32; 3]> {
    with_bot(|bot| bot.brk.take())
}

/// One client tick. `stand` is true when the feet can occupy that block.
pub fn tick(feet: [f64; 3], stand: &dyn Fn([i32; 3]) -> bool, find: &dyn Fn(&str) -> Option<[i32; 3]>) -> Look {
    with_bot(|bot| step(bot, feet, stand, find))
}

#[derive(Clone, Copy, Debug)]
pub struct Look {
    pub yaw: f32,
    pub pitch: f32,
    pub drive: bool,
}

fn step(bot: &mut Bot, feet: [f64; 3], stand: &dyn Fn([i32; 3]) -> bool, find: &dyn Fn(&str) -> Option<[i32; 3]>) -> Look {
    let mut look = Look {
        yaw: 0.0,
        pitch: 12.0,
        drive: false,
    };
    let mut mv = Move::default();
    if bot.goal == Goal::Idle {
        *MOVE.lock().unwrap_or_else(|p| p.into_inner()) = mv;
        return look;
    }
    let here = [feet[0].floor() as i32, feet[1].floor() as i32, feet[2].floor() as i32];
    bot.replan = bot.replan.saturating_sub(1);
    let target = match &bot.goal {
        Goal::Idle => None,
        Goal::Goto(at) => Some(*at),
        Goal::Mine(id) => {
            if bot.brk.is_none() || bot.replan == 0 {
                find(id).map(|ore| {
                    bot.brk = Some(ore);
                    stand_near(ore, stand).unwrap_or(ore)
                })
            } else {
                bot.brk.map(|ore| stand_near(ore, stand).unwrap_or(ore))
            }
        }
    };
    let Some(target) = target else {
        bot.status = "рядом такого блока нет".into();
        bot.path.clear();
        *MOVE.lock().unwrap_or_else(|p| p.into_inner()) = mv;
        return look;
    };
    if Goal::Goto(target) == bot.goal || matches!(bot.goal, Goal::Mine(_)) {
        let dx = feet[0] - (target[0] as f64 + 0.5);
        let dz = feet[2] - (target[2] as f64 + 0.5);
        if matches!(bot.goal, Goal::Goto(_)) && dx * dx + dz * dz < 0.45 && (feet[1] - target[1] as f64).abs() < 1.2
        {
            bot.status = "на месте".into();
            bot.goal = Goal::Idle;
            bot.path.clear();
            *MOVE.lock().unwrap_or_else(|p| p.into_inner()) = mv;
            return look;
        }
    }
    if let Some(ore) = bot.brk {
        let reach = dist(feet, center(ore));
        if reach < 4.2 && matches!(bot.goal, Goal::Mine(_)) {
            look.yaw = yaw_to(feet, center(ore));
            look.pitch = pitch_to(feet, center(ore));
            look.drive = true;
            bot.status = format!("копаю {} {} {}", ore[0], ore[1], ore[2]);
            mv.on = true;
            *MOVE.lock().unwrap_or_else(|p| p.into_inner()) = mv;
            return look;
        }
    }
    if bot.path.len() < 2 || bot.replan == 0 || !bot.path.iter().any(|p| *p == here) {
        match route(here, target, stand) {
            Some(path) => {
                bot.path = path;
                bot.replan = 15;
                bot.status = format!("путь {} блоков", bot.path.len().saturating_sub(1));
            }
            None => {
                bot.path.clear();
                bot.status = "пути нет (чанк не загружен или стена)".into();
                *MOVE.lock().unwrap_or_else(|p| p.into_inner()) = mv;
                return look;
            }
        }
    }
    while bot.path.len() >= 2 {
        let n = bot.path[1];
        let dx = feet[0] - (n[0] as f64 + 0.5);
        let dz = feet[2] - (n[2] as f64 + 0.5);
        if dx * dx + dz * dz < 0.2 && (feet[1] - n[1] as f64).abs() < 0.9 {
            bot.path.remove(0);
        } else {
            break;
        }
    }
    let next = bot.path.get(1).copied().unwrap_or(target);
    let at = [next[0] as f64 + 0.5, next[1] as f64, next[2] as f64 + 0.5];
    look.yaw = yaw_to(feet, at);
    look.pitch = 12.0;
    look.drive = true;
    let moved = (feet[0] - bot.stuck[0]).hypot(feet[2] - bot.stuck[2]);
    if moved < 0.04 {
        bot.stuck_ticks = bot.stuck_ticks.saturating_add(1);
    } else {
        bot.stuck_ticks = 0;
        bot.stuck = feet;
    }
    mv.forward = 1.0;
    mv.jump = next[1] > here[1] || bot.stuck_ticks > 12;
    if bot.stuck_ticks > 40 {
        bot.replan = 0;
        bot.stuck_ticks = 0;
    }
    mv.on = true;
    *MOVE.lock().unwrap_or_else(|p| p.into_inner()) = mv;
    look
}

fn center(b: [i32; 3]) -> [f64; 3] {
    [b[0] as f64 + 0.5, b[1] as f64 + 0.5, b[2] as f64 + 0.5]
}

fn dist(a: [f64; 3], b: [f64; 3]) -> f64 {
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

pub fn yaw_to(from: [f64; 3], to: [f64; 3]) -> f32 {
    let dx = to[0] - from[0];
    let dz = to[2] - from[2];
    (-dx).atan2(dz).to_degrees() as f32
}

pub fn pitch_to(from: [f64; 3], to: [f64; 3]) -> f32 {
    let dx = to[0] - from[0];
    let dy = to[1] - (from[1] + 1.62);
    let dz = to[2] - from[2];
    let flat = (dx * dx + dz * dz).sqrt();
    (-dy).atan2(flat).to_degrees() as f32
}

fn stand_near(ore: [i32; 3], stand: &dyn Fn([i32; 3]) -> bool) -> Option<[i32; 3]> {
    let mut best: Option<[i32; 3]> = None;
    let mut best_d = i32::MAX;
    for y in (ore[1] - 1)..=(ore[1] + 1) {
        for dz in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dz == 0 && y == ore[1] {
                    continue;
                }
                let at = [ore[0] + dx, y, ore[2] + dz];
                if !stand(at) {
                    continue;
                }
                let d = dx.abs() + dz.abs() + (y - ore[1]).abs();
                if d < best_d {
                    best_d = d;
                    best = Some(at);
                }
            }
        }
    }
    best
}

#[derive(Clone, Copy)]
struct Node {
    at: [i32; 3],
    g: i32,
    f: i32,
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.f == other.f
    }
}
impl Eq for Node {}
impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.f.cmp(&self.f).then_with(|| other.g.cmp(&self.g))
    }
}

const MAX_NODES: usize = 6000;

/// Feet cells from `start` to `goal`. Both must be standable.
pub fn route(start: [i32; 3], goal: [i32; 3], stand: &dyn Fn([i32; 3]) -> bool) -> Option<Vec<[i32; 3]>> {
    if start == goal {
        return Some(vec![start]);
    }
    if !stand(start) || !stand(goal) {
        return None;
    }
    let mut open = BinaryHeap::new();
    let mut best: HashMap<[i32; 3], i32> = HashMap::new();
    let mut came: HashMap<[i32; 3], [i32; 3]> = HashMap::new();
    open.push(Node {
        at: start,
        g: 0,
        f: heuristic(start, goal),
    });
    best.insert(start, 0);
    let mut seen = 0usize;
    while let Some(Node { at, g, .. }) = open.pop() {
        if best.get(&at).is_some_and(|known| *known < g) {
            continue;
        }
        if at == goal {
            return Some(rebuild(came, start, goal));
        }
        seen += 1;
        if seen > MAX_NODES {
            return None;
        }
        for (nx, nz) in NEIGHBORS {
            let Some(feet) = landing(at, nx, nz, stand) else {
                continue;
            };
            if !corner_ok(at, feet, stand) {
                continue;
            }
            let step = 10 + (feet[1] - at[1]).abs() * 4 + if nx != 0 && nz != 0 { 4 } else { 0 };
            let ng = g + step;
            if best.get(&feet).is_some_and(|known| *known <= ng) {
                continue;
            }
            came.insert(feet, at);
            best.insert(feet, ng);
            open.push(Node {
                at: feet,
                g: ng,
                f: ng + heuristic(feet, goal),
            });
        }
    }
    None
}

const NEIGHBORS: [(i32, i32); 8] = [
    (1, 0),
    (-1, 0),
    (0, 1),
    (0, -1),
    (1, 1),
    (1, -1),
    (-1, 1),
    (-1, -1),
];

fn landing(from: [i32; 3], dx: i32, dz: i32, stand: &dyn Fn([i32; 3]) -> bool) -> Option<[i32; 3]> {
    let x = from[0] + dx;
    let z = from[2] + dz;
    for dy in [0, 1, -1, -2, -3] {
        let at = [x, from[1] + dy, z];
        if stand(at) {
            return Some(at);
        }
    }
    None
}

fn corner_ok(from: [i32; 3], to: [i32; 3], stand: &dyn Fn([i32; 3]) -> bool) -> bool {
    let dx = to[0] - from[0];
    let dz = to[2] - from[2];
    if dx == 0 || dz == 0 {
        return true;
    }
    let side_x = stand([from[0] + dx, from[1], from[2]]) || stand([from[0] + dx, to[1], from[2]]);
    let side_z = stand([from[0], from[1], from[2] + dz]) || stand([from[0], to[1], from[2] + dz]);
    side_x && side_z
}

fn heuristic(a: [i32; 3], b: [i32; 3]) -> i32 {
    let dx = (a[0] - b[0]).abs();
    let dz = (a[2] - b[2]).abs();
    let dy = (a[1] - b[1]).abs();
    dx.max(dz) * 10 + dx.min(dz) * 4 + dy * 4
}

fn rebuild(came: HashMap<[i32; 3], [i32; 3]>, start: [i32; 3], goal: [i32; 3]) -> Vec<[i32; 3]> {
    let mut out = vec![goal];
    let mut cur = goal;
    while cur != start {
        cur = came[&cur];
        out.push(cur);
        if out.len() > MAX_NODES {
            break;
        }
    }
    out.reverse();
    out
}

/// A block the player can stand in: two air blocks of body and a solid floor.
pub fn feet_ok(at: [i32; 3], solid: &dyn Fn([i32; 3]) -> bool) -> bool {
    !solid(at) && !solid([at[0], at[1] + 1, at[2]]) && solid([at[0], at[1] - 1, at[2]])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat(at: [i32; 3]) -> bool {
        at[1] == 64 && at[0].abs() < 40 && at[2].abs() < 40
    }

    #[test]
    fn walks_across_flat_ground() {
        let path = route([-2, 64, 0], [3, 64, 1], &flat).unwrap();
        assert_eq!(*path.first().unwrap(), [-2, 64, 0]);
        assert_eq!(*path.last().unwrap(), [3, 64, 1]);
    }

    #[test]
    fn refuses_a_walled_off_goal() {
        let stand = |at: [i32; 3]| flat(at) && at[0] != 0;
        assert!(route([-2, 64, 0], [2, 64, 0], &stand).is_none());
    }

    #[test]
    fn steps_up_one() {
        let stand = |at: [i32; 3]| {
            if at[0] >= 2 {
                at[1] == 65 && at[2].abs() < 5 && at[0] < 8
            } else {
                flat(at)
            }
        };
        let path = route([0, 64, 0], [4, 65, 0], &stand).unwrap();
        assert!(path.iter().any(|p| p[1] == 65));
    }

    #[test]
    fn yaw_faces_south() {
        let yaw = yaw_to([0.0, 64.0, 0.0], [0.0, 64.0, 2.0]);
        assert!(yaw.abs() < 0.01, "{yaw}");
    }

    #[test]
    fn goto_parses() {
        let lines = handle("b goto 10 70 -3").unwrap();
        assert!(lines[0].contains("10"));
        assert_eq!(with_bot(|b| b.goal.clone()), Goal::Goto([10, 70, -3]));
        handle("b stop").unwrap();
    }
}
