use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::render::{Texture, TextureCreator, Canvas};
use sdl2::surface::Surface;
use sdl2::ttf::Sdl2TtfContext;
use sdl2::video::{WindowContext, Window};
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;
use sdl2::image::LoadTexture;
use rand::Rng;

const WIDTH: u32 = 1200;
const HEIGHT: u32 = 1000;

const TWENTY_ONE: usize = 21;
const CASINO_STOP_SCORE: usize = 17;

const WIN_NAME: &str = "BlackJack";

const TAKE_ANOTHER_CARD_TEXT: &str = "Press F to take another card";
const STOP_TAKING_CARDS_TEXT: &str = "Press E to stay with cards currently in hand";

const PLAYER_WINS_TEXT: &str = "Player wins!";
const CASINO_WINS_TEXT: &str = "Casino wins!";
const ITS_A_TIE_TEXT: &str = "It's a tie!";
const N_TO_RESTART_THE_GAME: &str = "Press N to restart the game";

#[derive(Clone, Copy)]
enum CardType {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace
}

impl CardType {
    fn iterator() -> impl Iterator<Item = CardType> {
        return [CardType::Two,
        CardType::Three,
        CardType::Four,
        CardType::Five,
        CardType::Six,
        CardType::Seven,
        CardType::Eight,
        CardType::Nine,
        CardType::Ten,
        CardType::Jack,
        CardType::Queen,
        CardType::King,
        CardType::Ace].iter().copied();
    }

    fn get_score(&self) -> usize {
        return match self {
            CardType::Two => 2,
            CardType::Three => 3,
            CardType::Four => 4,
            CardType::Five => 5,
            CardType::Six => 6,
            CardType::Seven => 7,
            CardType::Eight => 8,
            CardType::Nine => 9,
            CardType::Ten => 10,
            CardType::Jack | CardType::Queen | CardType::King => 10,
            CardType::Ace => 11,
        } 
    }

    fn get_string_name(&self) -> String {
        return match self {
            CardType::Two => "2".to_string(),
            CardType::Three => "3".to_string(),
            CardType::Four => "4".to_string(),
            CardType::Five => "5".to_string(),
            CardType::Six => "6".to_string(),
            CardType::Seven => "7".to_string(),
            CardType::Eight => "8".to_string(),
            CardType::Nine => "9".to_string(),
            CardType::Ten => "10".to_string(),
            CardType::Jack => "jack".to_string(),
            CardType::Queen => "queen".to_string(), 
            CardType::King => "king".to_string(),
            CardType::Ace => "ace".to_string(),
        } 
    }
}

struct TextureManager<'a> {
    cache: HashMap<String, Rc<Texture<'a>>>,
    loader: &'a TextureCreator<WindowContext>
}

impl <'a> TextureManager<'a> {
    fn load_texture(&mut self, path: &str) -> &Rc<Texture> {
        if  self.cache.contains_key(path) {
            return &self.cache[path];
        }

        self.cache.insert(path.to_string(), Rc::new(self.loader.load_texture(path).unwrap()));
        return &self.cache[path];
    }

    fn load_texture_from_surface(&mut self, path: &str, surface: Surface) {
        self.cache.insert(path.to_string(), Rc::new(self.loader.create_texture_from_surface(surface).unwrap()));
    }

    fn new(loader: &'a TextureCreator<WindowContext>) -> TextureManager<'a> {
        return TextureManager {
            cache: HashMap::<String, Rc<Texture<'a>>>::new(),
            loader: loader
        };
    }
}

#[derive(Clone, Copy)]
enum CardSuit {
    Clubs,
    Diamonds,
    Hearts,
    Spades
}

impl CardSuit {
    fn iterator() -> impl Iterator<Item = CardSuit> {
        return [
            CardSuit::Clubs,
            CardSuit::Diamonds,
            CardSuit::Hearts,
            CardSuit::Spades,
        ].iter().copied();
    }

    fn get_string_name(&self) -> String {
        return match self {
            CardSuit::Clubs => "clubs".to_string(),
            CardSuit::Diamonds => "diamonds".to_string(),
            CardSuit::Hearts => "hearts".to_string(),
            CardSuit::Spades => "spades".to_string(),
        };
    }
}

struct Card {
    card_type: CardType,
    _card_suit: CardSuit,
    path: String
}

enum Winner {
    Player,
    Casino,
    Tie
}

enum GameStatus {
    Uninitialized,
    AwaitingPlayerDecision,
    GameOver(Winner),
    PlayerStopedTakingCards
}

struct Game<'a> {
    status: GameStatus,
    deck: Vec<Card>,
    used_cards: Vec<usize>,
    player_hand: Vec<usize>,
    casino_hand: Vec<usize>,
    canvas: Canvas<Window>,
    texture_manager: TextureManager<'a>
}

impl <'a> Game<'a> {
    fn new(deck: Vec<Card>, canvas: Canvas<Window>, texture_manager: TextureManager<'a>) -> Game<'a> {
        let game = Game {
            status: GameStatus::Uninitialized,
            deck: deck,
            used_cards: Vec::<usize>::new(),
            player_hand: Vec::<usize>::new(),
            casino_hand: Vec::<usize>::new(),
            canvas: canvas,
            texture_manager: texture_manager
        };
        
        return game;
    }

    fn exec_cycle(&mut self,  keycodes: &Vec<Keycode>) {
        self.canvas.set_draw_color(Color::RGB(25, 120, 50));
        self.canvas.clear();

        match self.status {
            GameStatus::Uninitialized => self.exec_game_uninitialized(),
            GameStatus::AwaitingPlayerDecision => self.exec_game_awaiting_player_decision(keycodes),
            GameStatus::GameOver(_) => self.exec_game_game_over(keycodes),
            GameStatus::PlayerStopedTakingCards => self.exec_game_player_stopped_taking_cards()
        }

        self.render_hands();
        self.canvas.present();
    }

    fn exec_game_uninitialized(&mut self) {
        let mut random_card = self.get_random_card().unwrap();
        self.casino_hand.push(random_card);

        random_card = self.get_random_card().unwrap();
        self.player_hand.push(random_card);

        random_card = self.get_random_card().unwrap();
        self.player_hand.push(random_card);

        let player_score = self.calculate_hand_score(&self.player_hand);

        if player_score == TWENTY_ONE {
            self.status = GameStatus::PlayerStopedTakingCards;
        } else {
            self.status = GameStatus::AwaitingPlayerDecision;
        }
    }

    fn exec_game_awaiting_player_decision(&mut self, keycodes: &Vec<Keycode>) {
        self.canvas.copy(
            &self.texture_manager.load_texture(TAKE_ANOTHER_CARD_TEXT), None, 
            Rect::new(0, HEIGHT as i32 - 160,WIDTH, 80)).unwrap();
        self.canvas.copy(
            &self.texture_manager.load_texture(STOP_TAKING_CARDS_TEXT), None, 
            Rect::new(0, HEIGHT as i32 - 80,WIDTH, 80)).unwrap();

        if keycodes.contains(&Keycode::F) {
            let random_card = self.get_random_card().unwrap();
            self.player_hand.push(random_card);

            let player_score = self.calculate_hand_score(&self.player_hand);
            if player_score > TWENTY_ONE {
                self.status = GameStatus::GameOver(Winner::Casino);   
            } else if player_score == TWENTY_ONE {
                self.status = GameStatus::PlayerStopedTakingCards; 
            }
        } else if keycodes.contains(&Keycode::E) {
            self.status = GameStatus::PlayerStopedTakingCards;
        }
    }

    fn exec_game_game_over(&mut self, keycodes: &Vec<Keycode>) {
        let mut winner = &Winner::Tie;
        match &self.status {
            GameStatus::GameOver(win) => {
                winner = win;
            },
            _ => return,
        }

        match winner {
            Winner::Casino => self.canvas.copy(
                &self.texture_manager.load_texture(CASINO_WINS_TEXT), None, 
                Rect::new(0, HEIGHT as i32 - 160,WIDTH, 80)).unwrap(),
            Winner::Player => self.canvas.copy(
                &self.texture_manager.load_texture(PLAYER_WINS_TEXT), None, 
                Rect::new(0, HEIGHT as i32 - 160,WIDTH, 80)).unwrap(),
            Winner::Tie => self.canvas.copy(
                &self.texture_manager.load_texture(ITS_A_TIE_TEXT), None, 
                Rect::new(0, HEIGHT as i32 - 160,WIDTH, 80)).unwrap(),
        }

        self.canvas.copy(
            &self.texture_manager.load_texture(N_TO_RESTART_THE_GAME), None, 
            Rect::new(0, HEIGHT as i32 - 80,WIDTH, 80)).unwrap();

        if keycodes.contains(&Keycode::N) {
            self.status = GameStatus::Uninitialized;
            self.used_cards = Vec::<usize>::new();
            self.player_hand = Vec::<usize>::new();
            self.casino_hand = Vec::<usize>::new();
        }
    }

    fn exec_game_player_stopped_taking_cards(&mut self) {
        let player_score = self.calculate_hand_score(&self.player_hand);
        let mut casino_score = self.calculate_hand_score(&self.casino_hand);

        while casino_score < CASINO_STOP_SCORE && casino_score <= player_score {
            let random_card = self.get_random_card().unwrap();
            self.casino_hand.push(random_card);

            casino_score = self.calculate_hand_score(&self.casino_hand);
        }

        if casino_score > TWENTY_ONE {
            self.status = GameStatus::GameOver(Winner::Player);
        } else if casino_score > player_score {
            self.status = GameStatus::GameOver(Winner::Casino);
        } else if casino_score < player_score {
            self.status = GameStatus::GameOver(Winner::Player);
        } else {
            self.status = GameStatus::GameOver(Winner::Tie);
        }
    }

    fn render_hands(&mut self) {
        for (idx, card) in (&self.casino_hand).into_iter().enumerate() {
            let text_path = &self.deck[*card].path;
            let text = self.texture_manager.load_texture(&text_path);
            self.canvas.copy(&text, None, Rect::new(0 + (idx as i32 * 100), 0, 100, 150)).unwrap();
        }

        for (idx, card) in (&self.player_hand).into_iter().enumerate() {
            let text_path = &self.deck[*card].path;
            let text = self.texture_manager.load_texture(&text_path);
            self.canvas.copy(&text, None, Rect::new(0 + (idx as i32 * 100), 500,100, 150)).unwrap();
        }
    }

    fn get_random_card(&mut self) -> Option<usize> {
        if self.deck.len() <= self.used_cards.len() {
            return None;
        }
    
        let mut rng = rand::thread_rng();
        let mut index = rng.gen_range(0..self.deck.len());
    
        while self.used_cards.contains(&index) {
            index = rng.gen_range(0..self.deck.len());
        }
    
        self.used_cards.push(index);
        
        return Some(index);
    }

    fn calculate_hand_score(&self, hand: &Vec<usize>) -> usize {
        let mut result = 0;
        for card in hand {
            let card_score = self.deck[*card].card_type.get_score();
            result += card_score;
        }
    
        return result;
    }
}

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
 
    let window = video_subsystem.window(WIN_NAME, WIDTH, HEIGHT)
        .position_centered()
        .build()
        .unwrap();

    let ttf_context = sdl2::ttf::init().unwrap();
    let canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();
    let deck = get_deck();
    let mut texture_manager = TextureManager::new(&texture_creator);

    init_font_textures(&ttf_context, &mut texture_manager);

    let mut game = Game::new(deck, canvas, texture_manager);
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        let mut pressed_keycodes = Vec::<Keycode>::new();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown { keycode: Some(keycode), .. } => {
                    pressed_keycodes.push(keycode);
                },
                _ => {}
            }
        }

        game.exec_cycle(&pressed_keycodes);

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}

fn init_font_textures(ttf_context: &Sdl2TtfContext, texture_manager: &mut TextureManager) {
    let font = ttf_context
        .load_font("./assets/fonts/opensans/OpenSans-Regular.ttf", 128)
        .unwrap()
    ;

    for str in [
        TAKE_ANOTHER_CARD_TEXT, PLAYER_WINS_TEXT, 
        CASINO_WINS_TEXT, ITS_A_TIE_TEXT, 
        N_TO_RESTART_THE_GAME, STOP_TAKING_CARDS_TEXT] {
        let surface = font
            .render(str)
            .blended(Color::RGB(255, 255, 255))
            .unwrap()
        ;

        texture_manager.load_texture_from_surface(str, surface);
    }
}

fn get_deck() -> Vec::<Card> {
    let mut vec = Vec::<Card>::new();
    for tp in CardType::iterator() {
        for suit in CardSuit::iterator() {
            let texture_path = tp.get_string_name() + "_of_" + suit.get_string_name().as_str() + ".png";
            vec.push(Card { card_type: tp, _card_suit: suit, path: "assets/cards/".to_owned() + texture_path.as_str() })
        }
    }

    return vec
}