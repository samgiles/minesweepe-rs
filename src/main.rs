use iced::executor;
use iced::{
    canvas::{self, Cache, Canvas, Cursor, Event, Geometry, Text},
    mouse, Application, Color, Column, Command, Container, Element, HorizontalAlignment, Length,
    Point, Rectangle, Settings, Size, Subscription, VerticalAlignment,
};
use rand::thread_rng;
use std::collections::HashSet;
use std::fmt;

fn main() -> iced::Result {
    Minesweeper::run(Settings {
        antialiasing: true,
        ..Settings::default()
    })
}

struct Minesweeper {
    grid: UIGrid,
}

#[derive(Debug)]
enum UIMessage {
    Reveal(usize, usize),
    Flag(usize, usize),
}

impl Application for Minesweeper {
    type Message = UIMessage;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        let board = Board::new(40, 40, 50);
        (
            Self {
                grid: UIGrid {
                    board,
                    grid_cache: Cache::default(),
                },
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "Minesweeper".into()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        println!("Handling message: {:?}", message);
        self.grid.update(message);
        Command::none()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::none()
    }

    fn view(&mut self) -> Element<Self::Message> {
        let content = Column::new().push(self.grid.view());

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

struct UIGrid {
    board: Board,

    grid_cache: Cache,
}

impl UIGrid {
    pub fn view<'a>(&'a mut self) -> Element<'a, UIMessage> {
        Canvas::new(self)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    pub fn update(&mut self, message: UIMessage) {
        match message {
            UIMessage::Reveal(row, col) => {
                self.board.reveal_cell(row, col);
                self.grid_cache.clear();
            }
            UIMessage::Flag(row, col) => {
                self.board.flag_cell(row, col);
                self.grid_cache.clear();
            }
        }
    }

    pub fn project(&self, position: Point, size: Size) -> Point {
        let cell_w = size.width / self.board.width() as f32;
        let cell_h = size.height / self.board.height() as f32;
        Point::new(position.x / cell_w, position.y / cell_h)
    }
}

impl<'a> canvas::Program<UIMessage> for UIGrid {
    fn update(&mut self, event: Event, bounds: Rectangle, cursor: Cursor) -> Option<UIMessage> {
        let cursor_position = cursor.position_in(&bounds)?;
        let cell = self.project(cursor_position, bounds.size());
        let col = cell.x as usize;
        let row = cell.y as usize;

        match event {
            Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::ButtonPressed(button) => match button {
                    mouse::Button::Left => Some(UIMessage::Reveal(row, col)),
                    mouse::Button::Right => Some(UIMessage::Flag(row, col)),
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        }
    }

    fn draw(&self, bounds: Rectangle, _cursor: Cursor) -> Vec<Geometry> {
        let cell_width = bounds.width / self.board.width() as f32;
        let cell_height = bounds.height / self.board.height() as f32;
        let color_border = Color::from_rgb8(70, 74, 83);

        let grid = self.grid_cache.draw(bounds.size(), |frame| {
            for row in 0..self.board.width() {
                for col in 0..self.board.height() {
                    let position_x = col as f32 * cell_width;
                    let position_y = row as f32 * cell_height;

                    let color = match self.board.get_cell_state(row, col) {
                        None => panic!("out of bounds"),
                        Some(CellState::Mine(true, false)) => Color::from_rgb8(255, 0, 0),
                        Some(CellState::Mine(false, true)) => Color::from_rgb8(255, 255, 122),
                        Some(CellState::Neighbours(true, value)) => {
                            if value != 0 {
                                let text = Text {
                                    color: Color::WHITE,
                                    size: 32.0,
                                    position: Point::new(position_x + 25.0, position_y + 30.0),
                                    horizontal_alignment: HorizontalAlignment::Right,
                                    vertical_alignment: VerticalAlignment::Bottom,
                                    ..Text::default()
                                };
                                frame.fill_text(Text {
                                    content: format!("{}", value),
                                    ..text
                                });
                            }
                            Color::from_rgb8(0, 0, 200)
                        }
                        _ => Color::from_rgb8(0, 200, 0),
                    };
                    frame.fill_rectangle(
                        Point::new(col as f32 * cell_width, row as f32 * cell_height),
                        Size::new(cell_width, cell_height),
                        color_border,
                    );
                    frame.fill_rectangle(
                        Point::new(
                            (col as f32 * cell_width) + 2.0,
                            (row as f32 * cell_height) + 2.0,
                        ),
                        Size::new(cell_width - 2.0, cell_height - 2.0),
                        color,
                    );
                }
            }
        });
        vec![grid]
    }

    fn mouse_interaction(&self, bounds: Rectangle, cursor: Cursor) -> mouse::Interaction {
        if cursor.is_over(&bounds) {
            return mouse::Interaction::Crosshair;
        }
        mouse::Interaction::default()
    }
}

#[derive(Copy, Clone, Debug)]
enum CellState {
    Mine(bool, bool),
    Neighbours(bool, u8),
}

const ALL_DIRECTIONS: [Direction; 8] = [
    Direction::N,
    Direction::E,
    Direction::S,
    Direction::W,
    Direction::NE,
    Direction::SE,
    Direction::SW,
    Direction::NW,
];

enum Direction {
    N,
    E,
    S,
    W,
    NE,
    SE,
    SW,
    NW,
}

impl Direction {
    fn offset(&self, (row, col): (usize, usize)) -> Option<(usize, usize)> {
        match self {
            Self::N if row > 0 => Some((row - 1, col)),
            Self::E => Some((row, col + 1)),
            Self::S => Some((row + 1, col)),
            Self::W if col > 0 => Some((row, col - 1)),
            Self::NE if row > 0 => Some((row - 1, col + 1)),
            Self::SE => Some((row + 1, col + 1)),
            Self::SW if col > 0 => Some((row + 1, col - 1)),
            Self::NW if row > 0 && col > 0 => Some((row - 1, col - 1)),
            _ => None,
        }
    }
}

impl fmt::Display for CellState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Mine(true, false) => write!(f, "X"),
            Self::Mine(false, true) => write!(f, "!"),
            Self::Neighbours(true, count) => write!(f, "{}", count),
            _ => write!(f, " "),
        }
    }
}

struct Board {
    grid: Vec<Vec<CellState>>,
    game_over: bool,
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for _ in 0..self.width() {
            write!(f, "----")?;
        }
        write!(f, "-\n")?;

        for row in &self.grid {
            for cell in row {
                write!(f, "| {} ", cell)?;
            }
            write!(f, "|\n-")?;
            for _ in row {
                write!(f, "----")?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

fn select_random_coords(coords: &mut [(usize, usize)], number: usize) -> &[(usize, usize)] {
    use rand::seq::SliceRandom;

    let mut rng = thread_rng();
    coords.partial_shuffle(&mut rng, number).0
}

impl Board {
    pub fn new(width: usize, height: usize, mines: usize) -> Self {
        assert!(width > 0);
        assert!(height > 0);

        let mut grid = Vec::with_capacity(height);
        let mut coords = Vec::with_capacity(height * width);

        for row in 0..height {
            grid.push(Vec::with_capacity(width));
            for col in 0..width {
                grid[row].push(CellState::Neighbours(false, 0));
                coords.push((col, row));
            }
        }

        let coords = select_random_coords(&mut coords, mines);

        for i in 0..mines {
            let (col, row) = coords[i];
            grid[row][col] = CellState::Mine(false, false);

            for direction in &ALL_DIRECTIONS {
                if let Some(offset) = direction.offset((row, col)) {
                    if in_bounds(width, height, offset) {
                        let (o_row, o_col) = offset;
                        let cell = grid[o_row].get_mut(o_col);
                        match cell {
                            Some(CellState::Neighbours(_, ref mut value)) => {
                                *value += 1;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        Self {
            grid,
            game_over: false,
        }
    }

    pub fn get_cell_state(&self, row: usize, col: usize) -> Option<CellState> {
        if !in_bounds(self.width(), self.height(), (row, col)) {
            return None;
        }

        Some(self.grid[row][col])
    }

    pub fn flag_cell(&mut self, row: usize, col: usize) {
        if !in_bounds(self.width(), self.height(), (row, col)) {
            println!("not in bounds");
            return;
        }

        let cell = &mut self.grid[row][col];

        match cell {
            CellState::Mine(false, flagged) if *flagged == false => {
                *flagged = true;
            }
            CellState::Mine(false, flagged) if *flagged == true => {
                *flagged = false;
            }
            _ => {}
        }
    }

    pub fn reveal_cell(&mut self, row: usize, col: usize) {
        if self.game_over {
            println!("game over");
            return;
        }

        if !in_bounds(self.width(), self.height(), (row, col)) {
            println!("not in bounds");
            return;
        }

        let cell = &mut self.grid[row][col];
        println!("{:?}", cell);

        match cell {
            CellState::Mine(ref mut revealed, false) if *revealed == false => {
                println!("Game over!");
                *revealed = true;
                self.game_over = true;
            }
            CellState::Mine(ref mut revealed, true) if *revealed == false => return,
            _ => {
                println!("revealing cells");
                let mut closed = HashSet::new();
                self.reveal_cell_dfs(row, col, &mut closed);
            }
        }
    }

    fn reveal_cell_dfs(&mut self, row: usize, col: usize, closed: &mut HashSet<(usize, usize)>) {
        if !in_bounds(self.width(), self.height(), (row, col)) {
            return;
        }

        if closed.contains(&(row, col)) {
            // might not need this.
            return;
        }

        closed.insert((row, col));
        let cell = &mut self.grid[row][col];

        match cell {
            CellState::Neighbours(true, _) => {}
            CellState::Neighbours(ref mut revealed, count) if *count > 0 => {
                *revealed = true;
            }
            CellState::Neighbours(ref mut revealed, 0) => {
                println!("revealing neighbours: ({}, {})", row, col);
                *revealed = true;

                for direction in &ALL_DIRECTIONS {
                    if let Some((o_row, o_col)) = direction.offset((row, col)) {
                        self.reveal_cell_dfs(o_row, o_col, closed);
                    }
                }
            }
            _ => {}
        }
    }

    pub fn height(&self) -> usize {
        self.grid.len()
    }

    pub fn width(&self) -> usize {
        self.grid.first().unwrap().len()
    }
}

pub fn in_bounds(width: usize, height: usize, (row, col): (usize, usize)) -> bool {
    row < height && col < width
}
