use colored::*;
use rand::{seq::IteratorRandom, Rng};
use std::collections::HashSet;
use std::fmt;

use crate::node::Node;
use crate::pos::{ConnectState, Connection, Line, Pos};

const DIRECTIONS: &'static [Pos] = &[Pos(-1, 0), Pos(1, 0), Pos(0, -1), Pos(0, 1)];
const RETRIES: i32 = 10;

const CONN_V: &'static str = "│";
const CONN_H: &'static str = "─";
const LOCK: &'static str = "╳";
const HINT_V: &'static str = "╎";
const HINT_H: &'static str = "╌";

const BOX_TOP: &'static str = "┌─┐";
const BOX_SIDE: &'static str = "│";
const BOX_BOT: &'static str = "└─┘";

const KEY: &'static str = "█";

#[rustfmt::skip]
const COLORS: &'static [Color] = &[
    Color::TrueColor { r: 76,  g: 175, b: 80  }, // #4caf50
    Color::TrueColor { r: 0,   g: 150, b: 136 }, // #009688
    Color::TrueColor { r: 33,  g: 150, b: 243 }, // #2196f3
    Color::TrueColor { r: 63,  g: 81,  b: 181 }, // #3f51b5
    Color::TrueColor { r: 103, g: 58,  b: 183 }, // #673ab7
    Color::TrueColor { r: 156, g: 39,  b: 176 }, // #9c27b0
    Color::TrueColor { r: 233, g: 30,  b: 99  }, // #e91e63
    Color::TrueColor { r: 244, g: 67,  b: 54  }, // #f44336
    Color::TrueColor { r: 255, g: 152, b: 0   }, // #ff9800
];

const GREY: Color = Color::TrueColor {
    r: 64,
    g: 64,
    b: 64,
};

trait BetterFormatter {
    fn write_string<T>(&mut self, string: T) -> fmt::Result
    where
        T: Into<String>;
    fn write_nl(&mut self) -> fmt::Result;
}

impl BetterFormatter for fmt::Formatter<'_> {
    fn write_string<T>(&mut self, string: T) -> fmt::Result
    where
        T: Into<String>,
    {
        self.write_str(string.into().as_str())
    }

    fn write_nl(&mut self) -> fmt::Result {
        self.write_str("\n")
    }
}

#[derive(Clone)]
pub struct NodeMap {
    nodes: Vec<Option<Node>>,
    connections: Vec<Connection>,
    width: isize,
    height: isize,
}

fn color(index: u8) -> Color {
    COLORS[index as usize]
}

impl NodeMap {
    fn write_row<F, T>(&self, f: &mut fmt::Formatter<'_>, row: isize, func: F) -> fmt::Result
    where
        F: Fn(Pos) -> T,
        T: Into<String>,
    {
        for col in 0..self.width {
            f.write_string(func(Pos(row, col)))?;
        }

        f.write_nl()?;

        Ok(())
    }

    fn vertical_conn(&self, f: &mut fmt::Formatter<'_>, row: isize, offset: Pos) -> fmt::Result {
        self.write_row(f, row, |pos| {
            let line = Line(pos, pos + offset);

            let arg = match self.get_connection(line) {
                Some(conn) => match conn.state {
                    ConnectState::Shortcut => HINT_V.color(GREY),
                    _ => ColoredString::from(CONN_V),
                },
                None => ColoredString::from(" "),
            };

            format!("  {}   ", arg)
        })
    }

    fn fmt_box_side(&self, pos: Pos, string: &str) -> String {
        let arg = match self.get_node(pos) {
            Some(node) => string.color(color(node.layer)),
            None => ColoredString::from("   "),
        };

        format!(" {}  ", arg)
    }

    fn fmt_conn<'a, F, T>(&self, pos: Pos, offset: Pos, func: F) -> String
    where
        F: Fn(&Connection) -> T,
        T: Into<ColoredString> + From<&'a str> + fmt::Display,
    {
        let line = Line(pos, pos + offset);

        let arg = match self.get_connection(line) {
            Some(conn) => func(conn),
            None => T::from(" "),
        };

        format!("{}", arg)
    }

    fn write_box_middle(&self, f: &mut fmt::Formatter<'_>, row: isize) -> fmt::Result {
        self.write_row(f, row, |pos| {
            let left = self.fmt_conn(pos, Pos(0, -1), |conn| match conn.state {
                ConnectState::Shortcut => HINT_H.color(GREY),
                _ => CONN_H.into(),
            });

            let middle = match self.get_node(pos) {
                Some(node) => {
                    let side = BOX_SIDE.color(color(node.layer));
                    let middle = match node.key {
                        Some(layer) => KEY.color(color(layer)),
                        None => ColoredString::from(" "),
                    };

                    format!("{}{}{}", side, middle, side)
                }
                None => "   ".to_string(),
            };

            let right = self.fmt_conn(pos, Pos(0, 1), |conn| match conn.state {
                ConnectState::Shortcut => HINT_H.color(GREY),
                _ => CONN_H.into(),
            });
            let lock = self.fmt_conn(pos, Pos(0, 1), |conn| match conn.state {
                ConnectState::Open => CONN_H.into(),
                ConnectState::Locked => LOCK.into(),
                ConnectState::Shortcut => HINT_H.color(GREY),
            });

            format!("{}{}{}{}", left, middle, right, lock)
        })
    }
}

impl fmt::Debug for NodeMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in 0..self.height {
            if row > 0 {
                self.vertical_conn(f, row, Pos(-1, 0))?;
            }

            self.write_row(f, row, |pos| self.fmt_box_side(pos, BOX_TOP))?;

            self.write_box_middle(f, row)?;

            self.write_row(f, row, |pos| self.fmt_box_side(pos, BOX_BOT))?;

            if row < self.height - 1 {
                self.vertical_conn(f, row, Pos(1, 0))?;

                self.write_row(f, row, |pos| {
                    let arg = self.fmt_conn(pos, Pos(1, 0), |conn| match conn.state {
                        ConnectState::Open => CONN_V.into(),
                        ConnectState::Locked => LOCK.into(),
                        ConnectState::Shortcut => HINT_V.color(GREY),
                    });

                    format!("  {}   ", arg)
                })?;
            }
        }

        Ok(())
    }
}

impl NodeMap {
    pub fn with_size(width: usize, height: usize) -> NodeMap {
        let nodes = std::iter::repeat_with(|| None)
            .take(width * height)
            .collect();

        let connections = vec![];

        NodeMap {
            nodes,
            connections,
            width: width as isize,
            height: height as isize,
        }
    }

    pub fn generate(width: isize, height: isize, layers: u8) -> Result<NodeMap, String> {
        let mut rng = rand::thread_rng();

        let mut initial_map = NodeMap::with_size(width as usize, height as usize);

        let initial_pos = Pos(
            rng.gen_range(0, initial_map.height),
            rng.gen_range(0, initial_map.width),
        );

        initial_map.set_node(initial_pos, Node::default());

        let initial_state = GeneratorState {
            prev_state: None,
            node_map: initial_map,
            pos_list: vec![initial_pos],
            amount: 1,
            layer: 0,
        };

        let mut retries_left = RETRIES;

        let mut state = initial_state;

        while state.layer < layers && retries_left >= 0 {
            match state.generate_next_state(&mut rng) {
                Ok(next_state) => state = next_state,
                _ => {
                    retries_left -= 1;

                    state = *state.prev_state.unwrap();
                }
            }
        }

        if retries_left < 0 {
            return Err(format!("Failed to generate after {} retries", 10));
        }

        state = state.generate_final_state(&mut rng).unwrap();

        Ok(state.node_map.to_owned())
    }

    fn get_node(&self, pos: Pos) -> Option<&Node> {
        let row = pos.0;
        let col = pos.1;
        match self.nodes.get((row * self.width + col) as usize) {
            Some(node) => match node {
                Some(node) => Some(node),
                None => None,
            },
            None => None,
        }
    }

    pub fn set_node(&mut self, pos: Pos, node: Node) {
        let row = pos.0;
        let col = pos.1;

        *self
            .nodes
            .get_mut((row * self.width + col) as usize)
            .unwrap() = Some(node);
    }

    pub fn get_connection(&self, conn: Line) -> Option<&Connection> {
        self.connections
            .iter()
            .find(|other_conn| conn == other_conn.line)
    }

    pub fn add_connection(&mut self, conn: Connection) {
        self.connections.push(conn);
    }
}

#[derive(Clone)]
struct GeneratorState {
    prev_state: Option<Box<GeneratorState>>,
    node_map: NodeMap,
    pos_list: Vec<Pos>,
    amount: usize,
    layer: u8,
}

impl GeneratorState {
    fn generate_next_state(
        &self,
        rng: &mut rand::rngs::ThreadRng,
    ) -> Result<GeneratorState, &'static str> {
        let mut next_state = self.clone();
        next_state.prev_state = Some(Box::new(self.clone()));

        next_state.layer += 1;
        next_state.amount = rng.gen_range(4, 8);

        for _ in 0..next_state.amount {
            next_state.generate_node(rng)?;
        }

        next_state.generate_connections();

        if self.prev_state.is_none() {
            return Ok(next_state);
        }

        let key_amount = next_state
            .pos_list
            .iter()
            .copied()
            .skip(self.pos_list.len())
            .flat_map(|pos| {
                self.pos_list
                    .iter()
                    .copied()
                    .skip(self.pos_list.len() - self.amount)
                    .map(move |other_pos| Line(pos, other_pos))
                    .filter_map(|line| next_state.node_map.get_connection(line))
            })
            .fold(0, |count, _| count + 1);

        for pos in self
            .pos_list
            .iter()
            .skip(1)
            .copied()
            .filter(|pos| !self.node_map.get_node(*pos).unwrap().key.is_some())
            .choose_multiple(rng, key_amount)
        {
            next_state.node_map.set_node(
                pos,
                Node {
                    key: Some(next_state.layer),
                    ..*next_state.node_map.get_node(pos).unwrap()
                },
            );
        }

        Ok(next_state)
    }

    fn generate_final_state(
        &self,
        rng: &mut rand::rngs::ThreadRng,
    ) -> Result<GeneratorState, &'static str> {
        let mut final_state = self.clone();
        final_state.prev_state = Some(Box::new(self.clone()));
        final_state.layer += 1;

        let end_pos = *final_state
            .available_spaces_with_skip(0)
            .iter()
            .choose(rng)
            .unwrap();

        final_state.pos_list.push(end_pos);
        final_state
            .node_map
            .set_node(end_pos, Node::with_layer(final_state.layer));

        final_state.node_map.add_connection(Connection {
            line: Line(
                end_pos,
                DIRECTIONS
                    .iter()
                    .map(|offset| end_pos + *offset)
                    .filter(|pos| final_state.node_map.get_node(*pos).is_some())
                    .choose(rng)
                    .unwrap(),
            ),
            state: ConnectState::Locked,
        });

        let key_pos = self
            .pos_list
            .iter()
            .skip(self.pos_list.len() - self.amount)
            .copied()
            .filter(|pos| !self.node_map.get_node(*pos).unwrap().key.is_some())
            .choose(rng)
            .unwrap();

        final_state.node_map.set_node(
            key_pos,
            Node {
                key: Some(final_state.layer),
                ..*final_state.node_map.get_node(key_pos).unwrap()
            },
        );

        Ok(final_state)
    }

    fn generate_node(&mut self, rng: &mut rand::rngs::ThreadRng) -> Result<(), &'static str> {
        let spaces = self.available_spaces();

        if spaces.len() == 0 {
            return Err("No available spaces!");
        }

        let random_index = rng.gen_range(0, spaces.len());
        let random_space = spaces.get(random_index).unwrap();

        self.pos_list.push(*random_space);

        self.node_map
            .set_node(*self.pos_list.last().unwrap(), Node::with_layer(self.layer));

        Ok(())
    }

    fn available_spaces(&self) -> Vec<Pos> {
        let prev_state = self.prev_state.as_ref().unwrap();

        self.available_spaces_with_skip(prev_state.pos_list.len() - prev_state.amount)
    }

    fn available_spaces_with_skip(&self, skip: usize) -> Vec<Pos> {
        self.pos_list
            .iter()
            .skip(skip)
            .flat_map(move |pos: &Pos| {
                DIRECTIONS
                    .iter()
                    .copied()
                    .filter_map(|offset| {
                        (*pos + offset).in_range(
                            0,
                            self.node_map.height - 1,
                            0,
                            self.node_map.width - 1,
                        )
                    })
                    .collect::<Vec<Pos>>()
            })
            .filter(|pos| !self.pos_list.contains(pos))
            .collect::<HashSet<Pos>>()
            .into_iter()
            .collect::<Vec<Pos>>()
    }

    fn generate_connections(&mut self) {
        let prev_state = self.prev_state.as_ref().unwrap();
        let state = match prev_state.prev_state {
            Some(_) => ConnectState::Locked,
            None => ConnectState::Open,
        };

        let prev_pos_list = prev_state
            .pos_list
            .iter()
            .skip(prev_state.pos_list.len() - prev_state.amount)
            .copied()
            .collect::<Vec<Pos>>();

        for pos in self.pos_list.iter().skip(prev_state.amount).copied() {
            for conn in DIRECTIONS
                .iter()
                .copied()
                .map(|offset| pos + offset)
                .filter(|pos| prev_pos_list.contains(pos))
                .map(|other_pos| Line(other_pos, pos))
                .map(|line| Connection { line, state })
                .collect::<Vec<Connection>>()
            {
                self.node_map.add_connection(conn);
            }
        }

        let curr_pos_list = self
            .pos_list
            .iter()
            .skip(prev_state.pos_list.len())
            .copied()
            .collect::<Vec<Pos>>();

        for pos in self.pos_list.iter().skip(prev_state.amount).copied() {
            for conn in DIRECTIONS
                .iter()
                .copied()
                .map(|offset| pos + offset)
                .filter(|pos| curr_pos_list.contains(pos))
                .map(|other_pos| Line(other_pos, pos))
                .map(|line| Connection {
                    line,
                    state: ConnectState::Open,
                })
                .collect::<HashSet<Connection>>()
                .into_iter()
                .collect::<Vec<Connection>>()
            {
                self.node_map.add_connection(conn);
            }
        }

        for pos in self.pos_list.iter().skip(prev_state.amount).copied() {
            for conn in DIRECTIONS
                .iter()
                .copied()
                .map(|offset| pos + offset)
                .filter(|pos| self.pos_list.contains(pos))
                .map(|other_pos| Line(other_pos, pos))
                .filter(|line| self.node_map.get_connection(*line).is_none())
                .map(|line| Connection {
                    line,
                    state: ConnectState::Shortcut,
                })
                .collect::<HashSet<Connection>>()
                .into_iter()
                .collect::<Vec<Connection>>()
            {
                self.node_map.add_connection(conn);
            }
        }
    }
}
