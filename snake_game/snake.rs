use crossterm::{
    cursor, event::{self, Event, KeyCode, KeyEvent},
    execute, queue,
    style::{Color, Print, SetBackgroundColor},
    terminal::{self, Clear, ClearType},
};
use rand::Rng;
use std::{
    collections::VecDeque,
    io::{stdout, Write},
    thread::sleep,
    time::{Duration, Instant},
};

const WIDTH: u16 = 20;
const HEIGHT: u16 = 10;

#[derive(Clone, Copy, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, PartialEq)]
struct Position {
    x: u16,
    y: u16,
}

struct Game {
    snake: VecDeque<Position>,
    direction: Direction,
    food: Position,
    game_over: bool,
}

impl Game {
    fn new() -> Self {
        let mut snake = VecDeque::new();
        snake.push_back(Position { x: 5, y: 5 });

        let food = Game::generate_food(&snake);
        Game {
            snake,
            direction: Direction::Right,
            food,
            game_over: false,
        }
    }

    fn generate_food(snake: &VecDeque<Position>) -> Position {
        let mut rng = rand::thread_rng();
        loop {
            let pos = Position {
                x: rng.gen_range(1..WIDTH - 1),
                y: rng.gen_range(1..HEIGHT - 1),
            };
            if !snake.contains(&pos) {
                return pos;
            }
        }
    }

    fn update(&mut self) {
        let head = *self.snake.front().unwrap();
        let new_head = match self.direction {
            Direction::Up => Position { x: head.x, y: head.y.saturating_sub(1) },
            Direction::Down => Position { x: head.x, y: head.y + 1 },
            Direction::Left => Position { x: head.x.saturating_sub(1), y: head.y },
            Direction::Right => Position { x: head.x + 1, y: head.y },
        };

        // Check for collisions
        if new_head.x == 0 || new_head.y == 0 || new_head.x >= WIDTH || new_head.y >= HEIGHT || self.snake.contains(&new_head) {
            self.game_over = true;
            return;
        }

        self.snake.push_front(new_head);

        if new_head == self.food {
            self.food = Game::generate_food(&self.snake);
        } else {
            self.snake.pop_back();
        }
    }

    fn change_direction(&mut self, new_direction: Direction) {
        // Prevent 180-degree turns
        match (self.direction, new_direction) {
            (Direction::Up, Direction::Down) => {}
            (Direction::Down, Direction::Up) => {}
            (Direction::Left, Direction::Right) => {}
            (Direction::Right, Direction::Left) => {}
            _ => self.direction = new_direction,
        }
    }

    fn draw(&self, stdout: &mut std::io::Stdout) {
        queue!(stdout, cursor::MoveTo(0, 0), Clear(ClearType::All)).unwrap();
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let pos = Position { x, y };
                if self.snake.contains(&pos) {
                    queue!(stdout, SetBackgroundColor(Color::Green), Print(" "), SetBackgroundColor(Color::Reset)).unwrap();
                } else if pos == self.food {
                    queue!(stdout, SetBackgroundColor(Color::Red), Print(" "), SetBackgroundColor(Color::Reset)).unwrap();
                } else {
                    queue!(stdout, Print(" ")).unwrap();
                }
            }
            queue!(stdout, Print("\r\n")).unwrap();
        }

        if self.game_over {
            queue!(stdout, Print("Game Over! Press 'q' to exit.\n")).unwrap();
        }

        stdout.flush().unwrap();
    }
}

fn main() {
    let mut stdout = stdout();
    terminal::enable_raw_mode().unwrap();
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide).unwrap();

    let mut game = Game::new();
    let mut last_frame = Instant::now();

    loop {
        while event::poll(Duration::from_millis(0)).unwrap() {
            if let Event::Key(KeyEvent { code, .. }) = event::read().unwrap() {
                match code {
                    KeyCode::Up => game.change_direction(Direction::Up),
                    KeyCode::Down => game.change_direction(Direction::Down),
                    KeyCode::Left => game.change_direction(Direction::Left),
                    KeyCode::Right => game.change_direction(Direction::Right),
                    KeyCode::Char('q') => {
                        game.game_over = true;
                        break;
                    }
                    _ => {}
                }
            }
        }

        if last_frame.elapsed() >= Duration::from_millis(120) {
            if !game.game_over {
                game.update();
            }
            game.draw(&mut stdout);
            last_frame = Instant::now();
        }

        if game.game_over {
            break;
        }

        sleep(Duration::from_millis(10));
    }

    // Clean up
    execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen).unwrap();
    terminal::disable_raw_mode().unwrap();
}

