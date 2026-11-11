use std::fmt;

pub const BOARD_SIZE: usize = 40;
pub const START_CASH: i32 = 1500;
pub const PASS_GO_CASH: i32 = 200;
pub const JAIL_POSITION: usize = 10;
pub const FREE_PARKING_POSITION: usize = 20;
pub const GO_TO_JAIL_POSITION: usize = 30;
pub const JAIL_FINE: i32 = 50;
pub const MAX_JAIL_TURNS: u8 = 3;

#[derive(Clone, Copy, Debug)]
pub struct DiceRoll {
    pub die1: u8,
    pub die2: u8,
}

impl DiceRoll {
    pub fn total(self) -> u8 {
        self.die1 + self.die2
    }

    pub fn is_double(self) -> bool {
        self.die1 == self.die2
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    Roll,
    BuyProperty,
    DeclineProperty,
    BuildHouse { position: usize },
    BuildHotel { position: usize },
    Trade(TradeOffer),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TradeOffer {
    pub from: usize,
    pub to: usize,
    pub cash_from_offer: i32,
    pub cash_to_offer: i32,
    pub give_positions: Vec<usize>,
    pub receive_positions: Vec<usize>,
}

#[derive(Clone, Debug)]
pub struct Observation {
    pub turn: u32,
    pub current_player: usize,
    pub free_parking_pot: i32,
    pub pending_purchase: Option<usize>,
    pub players: Vec<PlayerState>,
    pub lots: Vec<Option<LotState>>,
}

impl Observation {
    pub fn to_json(&self) -> String {
        let mut out = String::new();
        out.push('{');
        out.push_str(&format!("\"turn\":{},", self.turn));
        out.push_str(&format!("\"current_player\":{},", self.current_player));
        out.push_str(&format!("\"free_parking_pot\":{},", self.free_parking_pot));
        match self.pending_purchase {
            Some(pos) => out.push_str(&format!("\"pending_purchase\":{},", pos)),
            None => out.push_str("\"pending_purchase\":null,"),
        }
        out.push_str("\"players\":[");
        for (i, player) in self.players.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            out.push_str(&player.to_json());
        }
        out.push_str("],\"lots\":[");
        for (i, lot) in self.lots.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            match lot {
                Some(l) => out.push_str(&l.to_json()),
                None => out.push_str("null"),
            }
        }
        out.push_str("]}");
        out
    }
}

#[derive(Clone, Debug)]
pub struct StepResult {
    pub observation: Observation,
    pub reward: f32,
    pub done: bool,
    pub messages: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct PlayerState {
    pub name: String,
    pub position: usize,
    pub cash: i32,
    pub in_jail: bool,
    pub jail_turns_left: u8,
    pub active: bool,
}

impl PlayerState {
    fn new(name: String) -> Self {
        Self {
            name,
            position: 0,
            cash: START_CASH,
            in_jail: false,
            jail_turns_left: 0,
            active: true,
        }
    }

    fn to_json(&self) -> String {
        format!(
            "{{\"name\":\"{}\",\"position\":{},\"cash\":{},\"in_jail\":{},\"jail_turns_left\":{},\"active\":{}}}",
            escape_json(&self.name),
            self.position,
            self.cash,
            self.in_jail,
            self.jail_turns_left,
            self.active
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorGroup {
    Brown,
    LightBlue,
    Pink,
    Orange,
    Red,
    Yellow,
    Green,
    DarkBlue,
}

impl fmt::Display for ColorGroup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            ColorGroup::Brown => "Brown",
            ColorGroup::LightBlue => "Light Blue",
            ColorGroup::Pink => "Pink",
            ColorGroup::Orange => "Orange",
            ColorGroup::Red => "Red",
            ColorGroup::Yellow => "Yellow",
            ColorGroup::Green => "Green",
            ColorGroup::DarkBlue => "Dark Blue",
        };
        write!(f, "{name}")
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PropertyKind {
    Street {
        group: ColorGroup,
        house_cost: i32,
        base_rent: i32,
    },
    Airport {
        base_rent: i32,
    },
    Utility {
        base_rent: i32,
    },
}

#[derive(Clone, Debug)]
pub enum Square {
    Go,
    Property {
        name: &'static str,
        cost: i32,
        kind: PropertyKind,
    },
    Tax {
        name: &'static str,
        amount: i32,
    },
    Jail,
    FreeParking,
    Chance,
    CommunityChest,
    GoToJail,
}

impl Square {
    pub fn name(&self) -> &'static str {
        match self {
            Square::Go => "GO",
            Square::Property { name, .. } | Square::Tax { name, .. } => name,
            Square::Jail => "Jail",
            Square::FreeParking => "Free Parking",
            Square::Chance => "Chance",
            Square::CommunityChest => "Community Chest",
            Square::GoToJail => "Go To Jail",
        }
    }

    pub fn purchase_cost(&self) -> Option<i32> {
        match self {
            Square::Property { cost, .. } => Some(*cost),
            _ => None,
        }
    }

    pub fn is_buyable(&self) -> bool {
        self.purchase_cost().is_some()
    }

    pub fn is_street(&self) -> bool {
        matches!(self, Square::Property { kind: PropertyKind::Street { .. }, .. })
    }

    pub fn kind(&self) -> Option<PropertyKind> {
        match self {
            Square::Property { kind, .. } => Some(*kind),
            _ => None,
        }
    }

    pub fn board_label(&self) -> &'static str {
        match self {
            Square::Property { kind: PropertyKind::Street { group, .. }, .. } => match group {
                ColorGroup::Brown => "Brown",
                ColorGroup::LightBlue => "Light Blue",
                ColorGroup::Pink => "Pink",
                ColorGroup::Orange => "Orange",
                ColorGroup::Red => "Red",
                ColorGroup::Yellow => "Yellow",
                ColorGroup::Green => "Green",
                ColorGroup::DarkBlue => "Dark Blue",
            },
            Square::Property { kind: PropertyKind::Airport { .. }, .. } => "Airport",
            Square::Property { kind: PropertyKind::Utility { .. }, .. } => "Utility",
            _ => "",
        }
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Square::Go => write!(f, "GO"),
            Square::Property { name, .. } => write!(f, "{name}"),
            Square::Tax { name, amount } => write!(f, "{name} (${amount})"),
            Square::Jail => write!(f, "Jail"),
            Square::FreeParking => write!(f, "Free Parking"),
            Square::Chance => write!(f, "Chance"),
            Square::CommunityChest => write!(f, "Community Chest"),
            Square::GoToJail => write!(f, "Go To Jail"),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct LotState {
    pub owner: Option<usize>,
    pub houses: u8,
    pub hotel: bool,
}

impl LotState {
    fn to_json(&self) -> String {
        format!(
            "{{\"owner\":{},\"houses\":{},\"hotel\":{}}}",
            self.owner
                .map(|i| i.to_string())
                .unwrap_or_else(|| "null".to_string()),
            self.houses,
            self.hotel
        )
    }
}

#[derive(Clone, Debug)]
pub enum CardEffect {
    AdvanceTo(usize),
    MoveBy(i32),
    Earn(i32),
    Pay(i32),
    GoToJail,
    FreeParkingBonus(i32),
}

#[derive(Clone, Debug)]
pub struct Card {
    pub text: &'static str,
    pub effect: CardEffect,
}

#[derive(Clone, Debug)]
pub struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    pub fn seeded(seed: u64) -> Self {
        Self { state: seed.max(1) }
    }

    fn next_u32(&mut self) -> u32 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1);
        (self.state >> 32) as u32
    }

    pub fn range(&mut self, upper: usize) -> usize {
        if upper == 0 {
            0
        } else {
            (self.next_u32() as usize) % upper
        }
    }

    pub fn roll_die(&mut self) -> u8 {
        (self.range(6) as u8) + 1
    }
}

#[derive(Clone, Debug)]
struct PendingPurchase {
    position: usize,
}

pub struct GameState {
    pub board: Vec<Square>,
    pub lots: Vec<Option<LotState>>,
    pub players: Vec<PlayerState>,
    pub current_player: usize,
    pub turn: u32,
    pub free_parking_pot: i32,
    pub rng: SimpleRng,
    pub extra_turn_pending: bool,
    pending_purchase: Option<PendingPurchase>,
    chance_deck: Vec<Card>,
    chest_deck: Vec<Card>,
}

impl GameState {
    pub fn new(player_count: usize, seed: u64) -> Self {
        let player_count = player_count.clamp(2, 8);
        let board = default_board();
        let lots = board
            .iter()
            .map(|square| if square.is_buyable() { Some(LotState::default()) } else { None })
            .collect();
        let players = (0..player_count)
            .map(|i| PlayerState::new(format!("Player {}", i + 1)))
            .collect();

        Self {
            board,
            lots,
            players,
            current_player: 0,
            turn: 1,
            free_parking_pot: 0,
            rng: SimpleRng::seeded(seed),
            extra_turn_pending: false,
            pending_purchase: None,
            chance_deck: chance_cards(),
            chest_deck: community_chest_cards(),
        }
    }

    pub fn observation(&self) -> Observation {
        Observation {
            turn: self.turn,
            current_player: self.current_player,
            free_parking_pot: self.free_parking_pot,
            pending_purchase: self.pending_purchase.as_ref().map(|p| p.position),
            players: self.players.clone(),
            lots: self.lots.clone(),
        }
    }

    pub fn export_json(&self) -> String {
        self.observation().to_json()
    }

    pub fn legal_actions(&self) -> Vec<Action> {
        if self.pending_purchase.is_some() {
            vec![Action::BuyProperty, Action::DeclineProperty]
        } else {
            vec![
                Action::Roll,
                Action::Trade(TradeOffer {
                    from: self.current_player,
                    to: self.current_player,
                    cash_from_offer: 0,
                    cash_to_offer: 0,
                    give_positions: vec![],
                    receive_positions: vec![],
                }),
            ]
        }
    }

    pub fn pending_purchase_square(&self) -> Option<&Square> {
        self.pending_purchase
            .as_ref()
            .map(|p| &self.board[p.position])
    }

    pub fn pending_purchase_price(&self) -> Option<i32> {
        self.pending_purchase
            .as_ref()
            .and_then(|p| self.board[p.position].purchase_cost())
    }

    pub fn pending_purchase_owner(&self) -> Option<usize> {
        self.pending_purchase
            .as_ref()
            .and_then(|p| self.lots[p.position].as_ref().and_then(|l| l.owner))
    }

    pub fn step(&mut self, action: Action) -> StepResult {
        self.ensure_current_player_active();
        let player_index = self.current_player;
        let before_net_worth = self.net_worth(player_index);
        let mut messages = vec![format!("{}'s turn", self.players[player_index].name)];
        let was_current_player = self.current_player == player_index;

        match action {
            Action::Roll => {
                if self.pending_purchase.is_some() {
                    messages.push("Finish the pending purchase decision first.".to_string());
                } else {
                    self.handle_roll(player_index, &mut messages);
                }
            }
            Action::BuyProperty => {
                self.handle_buy(player_index, &mut messages);
            }
            Action::DeclineProperty => {
                self.handle_decline(&mut messages);
            }
            Action::BuildHouse { position } => {
                self.handle_build_house(player_index, position, &mut messages);
            }
            Action::BuildHotel { position } => {
                self.handle_build_hotel(player_index, position, &mut messages);
            }
            Action::Trade(offer) => {
                self.handle_trade(offer, &mut messages);
            }
        }

        let still_same_turn_owner = self.current_player == player_index && self.players[player_index].active;
        if was_current_player && still_same_turn_owner && self.pending_purchase.is_none() && !self.extra_turn_pending {
            self.advance_turn_if_needed();
        }

        let after_net_worth = self.net_worth(player_index);
        let reward = (after_net_worth - before_net_worth) as f32;
        self.turn += 1;

        StepResult {
            observation: self.observation(),
            reward,
            done: self.is_done(),
            messages,
        }
    }

    pub fn board_snapshot(&self) -> String {
        let mut out = String::new();
        for (idx, square) in self.board.iter().enumerate() {
            let occupants: Vec<String> = self
                .players
                .iter()
                .enumerate()
                .filter(|(_, p)| p.active && p.position == idx)
                .map(|(i, _)| format!("P{}", i + 1))
                .collect();

            let lot = self.lots[idx].as_ref();
            let owner = lot
                .and_then(|l| l.owner)
                .map(|i| format!(" owner=P{}", i + 1))
                .unwrap_or_default();
            let buildings = lot.map(building_label).unwrap_or_default();
            let marker = if occupants.is_empty() {
                String::new()
            } else {
                format!(" [{}]", occupants.join(", "))
            };

            let label = square.board_label();
            let label = if label.is_empty() {
                String::new()
            } else {
                label.to_string()
            };
            out.push_str(&format!("{:<10}: {:<22}{}{}{}\n", label, square.name(), marker, owner, buildings));
        }
        out
    }

    pub fn is_done(&self) -> bool {
        self.players.iter().filter(|p| p.active).count() <= 1
    }

    fn ensure_current_player_active(&mut self) {
        if self.players.get(self.current_player).map(|p| p.active).unwrap_or(false) {
            return;
        }
        self.current_player = self.next_active_player_from(self.current_player);
    }

    fn next_active_player_from(&self, start: usize) -> usize {
        let len = self.players.len();
        for offset in 1..=len {
            let idx = (start + offset) % len;
            if self.players[idx].active {
                return idx;
            }
        }
        start
    }

    fn advance_turn_if_needed(&mut self) {
        if self.pending_purchase.is_none() && !self.extra_turn_pending {
            self.current_player = self.next_active_player_from(self.current_player);
        }
    }

    fn handle_buy(&mut self, player_index: usize, messages: &mut Vec<String>) {
        let Some(pending) = self.pending_purchase.take() else {
            messages.push("There is nothing to buy right now.".to_string());
            return;
        };

        let square = &self.board[pending.position];
        let Some(cost) = square.purchase_cost() else {
            messages.push("That square cannot be bought.".to_string());
            return;
        };

        let Some(lot) = self.lots[pending.position].as_mut() else {
            messages.push("That square is not trackable as a lot.".to_string());
            return;
        };

        if lot.owner.is_some() {
            messages.push(format!("{} is already owned.", square.name()));
            return;
        }

        if self.players[player_index].cash < cost {
            messages.push(format!("Not enough cash to buy {}.", square.name()));
            return;
        }

        self.players[player_index].cash -= cost;
        lot.owner = Some(player_index);
        messages.push(format!("Bought {} for ${cost}.", square.name()));
    }

    fn handle_decline(&mut self, messages: &mut Vec<String>) {
        let Some(pending) = self.pending_purchase.take() else {
            messages.push("There is nothing to decline right now.".to_string());
            return;
        };

        messages.push(format!("Declined purchase of {}.", self.board[pending.position].name()));
    }

    fn handle_build_house(&mut self, player_index: usize, position: usize, messages: &mut Vec<String>) {
        let Some(square) = self.board.get(position) else {
            messages.push("Invalid board position.".to_string());
            return;
        };
        if !square.is_street() {
            messages.push(format!("{} cannot have houses.", square.name()));
            return;
        }
        if !self.player_owns_full_set(player_index, position) {
            messages.push(format!("{} does not own the full set on {}.", self.players[player_index].name, square.name()));
            return;
        }
        let Some(kind) = square.kind() else {
            messages.push("That property cannot be improved.".to_string());
            return;
        };
        let PropertyKind::Street { house_cost, .. } = kind else {
            messages.push(format!("{} cannot have houses.", square.name()));
            return;
        };
        let Some(lot) = self.lots[position].as_mut() else {
            messages.push("No lot data found.".to_string());
            return;
        };
        if lot.owner != Some(player_index) {
            messages.push("You do not own that property.".to_string());
            return;
        }
        if lot.hotel {
            messages.push("That property already has a hotel.".to_string());
            return;
        }
        if lot.houses >= 4 {
            messages.push("Build a hotel instead of more houses.".to_string());
            return;
        }
        if self.players[player_index].cash < house_cost {
            messages.push("Not enough cash to build a house.".to_string());
            return;
        }

        self.players[player_index].cash -= house_cost;
        lot.houses += 1;
        messages.push(format!("Built a house on {} for ${house_cost}.", square.name()));
    }

    fn handle_build_hotel(&mut self, player_index: usize, position: usize, messages: &mut Vec<String>) {
        let Some(square) = self.board.get(position) else {
            messages.push("Invalid board position.".to_string());
            return;
        };
        if !square.is_street() {
            messages.push(format!("{} cannot have a hotel.", square.name()));
            return;
        }
        if !self.player_owns_full_set(player_index, position) {
            messages.push(format!("{} does not own the full set on {}.", self.players[player_index].name, square.name()));
            return;
        }
        let Some(kind) = square.kind() else {
            messages.push("That property cannot be improved.".to_string());
            return;
        };
        let PropertyKind::Street { house_cost, .. } = kind else {
            messages.push(format!("{} cannot have a hotel.", square.name()));
            return;
        };
        let Some(lot) = self.lots[position].as_mut() else {
            messages.push("No lot data found.".to_string());
            return;
        };
        if lot.owner != Some(player_index) {
            messages.push("You do not own that property.".to_string());
            return;
        }
        if lot.hotel {
            messages.push("That property already has a hotel.".to_string());
            return;
        }
        if lot.houses < 4 {
            messages.push("Build 4 houses before building a hotel.".to_string());
            return;
        }
        if self.players[player_index].cash < house_cost {
            messages.push("Not enough cash to build a hotel.".to_string());
            return;
        }

        self.players[player_index].cash -= house_cost;
        lot.houses = 0;
        lot.hotel = true;
        messages.push(format!("Built a hotel on {} for ${house_cost}.", square.name()));
    }

    fn handle_trade(&mut self, offer: TradeOffer, messages: &mut Vec<String>) {
        if offer.from != self.current_player {
            messages.push("You can only trade on your own turn.".to_string());
            return;
        }
        if offer.from == offer.to {
            messages.push("Trade must involve two different players.".to_string());
            return;
        }
        if offer.to >= self.players.len() || !self.players[offer.to].active {
            messages.push("Target player is invalid or inactive.".to_string());
            return;
        }
        if self.players[offer.from].cash < offer.cash_from_offer {
            messages.push("You do not have enough cash for your side of the trade.".to_string());
            return;
        }
        if self.players[offer.to].cash < offer.cash_to_offer {
            messages.push("Other player does not have enough cash for their side of the trade.".to_string());
            return;
        }

        if !self.validate_trade_positions(offer.from, offer.to, &offer.give_positions, &offer.receive_positions) {
            messages.push("Trade failed because one or more lots are not owned by the expected players.".to_string());
            return;
        }

        self.players[offer.from].cash -= offer.cash_from_offer;
        self.players[offer.to].cash += offer.cash_from_offer;
        self.players[offer.to].cash -= offer.cash_to_offer;
        self.players[offer.from].cash += offer.cash_to_offer;

        for pos in &offer.give_positions {
            if let Some(lot) = self.lots[*pos].as_mut() {
                lot.owner = Some(offer.to);
            }
        }
        for pos in &offer.receive_positions {
            if let Some(lot) = self.lots[*pos].as_mut() {
                lot.owner = Some(offer.from);
            }
        }

        messages.push(format!(
            "Trade completed between {} and {}.",
            self.players[offer.from].name, self.players[offer.to].name
        ));
    }

    fn validate_trade_positions(&self, from: usize, to: usize, give_positions: &[usize], receive_positions: &[usize]) -> bool {
        give_positions.iter().all(|pos| {
            self.lots
                .get(*pos)
                .and_then(|lot| lot.as_ref())
                .map(|lot| lot.owner == Some(from))
                .unwrap_or(false)
        }) && receive_positions.iter().all(|pos| {
            self.lots
                .get(*pos)
                .and_then(|lot| lot.as_ref())
                .map(|lot| lot.owner == Some(to))
                .unwrap_or(false)
        })
    }

    fn handle_roll(&mut self, player_index: usize, messages: &mut Vec<String>) {
        self.extra_turn_pending = false;
        let dice = DiceRoll {
            die1: self.rng.roll_die(),
            die2: self.rng.roll_die(),
        };
        messages.push(format!(
            "Rolled {} and {} (total {}).",
            dice.die1,
            dice.die2,
            dice.total()
        ));

        if self.players[player_index].in_jail {
            self.handle_jail_roll(player_index, dice, messages);
            return;
        }

        messages.extend(self.move_by(player_index, i32::from(dice.total()), true));
        messages.extend(self.resolve_position(player_index));

        if self.players[player_index].active && !self.players[player_index].in_jail {
            self.extra_turn_pending = dice.is_double();
            if self.extra_turn_pending {
                messages.push("Doubles rolled: you get another roll.".to_string());
            }
        }
    }

    fn handle_jail_roll(&mut self, player_index: usize, dice: DiceRoll, messages: &mut Vec<String>) {
        let doubles = dice.is_double();
        if doubles {
            messages.push("Rolled doubles and got out of jail.".to_string());
            self.players[player_index].in_jail = false;
            self.players[player_index].jail_turns_left = 0;
            messages.extend(self.move_by(player_index, i32::from(dice.total()), true));
            messages.extend(self.resolve_position(player_index));
            self.extra_turn_pending = true;
            messages.push("Doubles rolled: you get another roll.".to_string());
            return;
        }

        if self.players[player_index].jail_turns_left > 0 {
            self.players[player_index].jail_turns_left -= 1;
        }

        if self.players[player_index].jail_turns_left > 0 {
            messages.push(format!(
                "No doubles. Jail turns left: {}.",
                self.players[player_index].jail_turns_left
            ));
            return;
        }

        messages.push(format!("No doubles on the last jail turn. Paid ${JAIL_FINE} to get out."));
        self.players[player_index].cash -= JAIL_FINE;
        self.resolve_cash_change(player_index, None, messages);
        if self.players[player_index].active {
            self.players[player_index].in_jail = false;
            self.players[player_index].jail_turns_left = 0;
            messages.extend(self.move_by(player_index, i32::from(dice.total()), true));
            messages.extend(self.resolve_position(player_index));
        }
    }

    fn move_by(&mut self, player_index: usize, steps: i32, can_collect_go: bool) -> Vec<String> {
        let old = self.players[player_index].position as i32;
        let board = BOARD_SIZE as i32;
        let next = old + steps;
        let wrapped = steps > 0 && next >= board;
        let new_pos = ((next % board) + board) % board;

        self.players[player_index].position = new_pos as usize;
        let mut messages = vec![format!("Moved from {} to {}.", old, self.players[player_index].position)];

        if can_collect_go && wrapped {
            self.players[player_index].cash += PASS_GO_CASH;
            messages.push(format!("Passed GO and collected ${PASS_GO_CASH}."));
        }

        messages
    }

    fn move_to(&mut self, player_index: usize, target: usize, can_collect_go: bool) -> Vec<String> {
        let old = self.players[player_index].position;
        let wrapped = can_collect_go && target < old;
        self.players[player_index].position = target;

        let mut messages = vec![format!("Moved from {} to {}.", old, target)];
        if wrapped {
            self.players[player_index].cash += PASS_GO_CASH;
            messages.push(format!("Passed GO and collected ${PASS_GO_CASH}."));
        }
        messages
    }

    fn resolve_position(&mut self, player_index: usize) -> Vec<String> {
        let mut messages = Vec::new();
        for _ in 0..8 {
            let square = self.board[self.players[player_index].position].clone();
            let (mut step_messages, recheck) = self.resolve_square(player_index, square);
            messages.append(&mut step_messages);
            if !recheck {
                break;
            }
        }
        self.resolve_cash_change(player_index, None, &mut messages);
        messages
    }

    fn resolve_square(&mut self, player_index: usize, square: Square) -> (Vec<String>, bool) {
        let mut messages = Vec::new();
        match square {
            Square::Go => {
                messages.push("Landed on GO.".to_string());
            }
            Square::Property { name, .. } => {
                let pos = self.players[player_index].position;
                if self.lots[pos].as_ref().and_then(|lot| lot.owner).is_none() {
                    self.pending_purchase = Some(PendingPurchase { position: pos });
                    if let Some(cost) = square.purchase_cost() {
                        messages.push(format!("Landed on {name}. Buy for ${cost} or decline."));
                    }
                } else if self.lots[pos].as_ref().and_then(|lot| lot.owner) == Some(player_index) {
                    messages.push(format!("You already own {name}."));
                } else {
                    let rent = self.calculate_rent(pos);
                    self.pay_rent(player_index, pos, rent, &mut messages);
                }
            }
            Square::Tax { name, amount } => {
                self.players[player_index].cash -= amount;
                messages.push(format!("Paid {name} of ${amount}."));
                self.free_parking_pot += amount;
                messages.push(format!("Free Parking pot is now ${}.", self.free_parking_pot));
                self.resolve_cash_change(player_index, None, &mut messages);
            }
            Square::Jail => {
                messages.push("Just visiting jail.".to_string());
            }
            Square::FreeParking => {
                if self.free_parking_pot > 0 {
                    let bonus = self.free_parking_pot;
                    self.players[player_index].cash += bonus;
                    self.free_parking_pot = 0;
                    messages.push(format!("Collected ${bonus} from Free Parking."));
                } else {
                    messages.push("Free Parking is empty.".to_string());
                }
            }
            Square::Chance => {
                let card = draw_card(&mut self.chance_deck, &mut self.rng, chance_cards);
                messages.push(format!("Chance card: {}", card.text));
                let (mut card_messages, recheck) = self.apply_card(player_index, card);
                messages.append(&mut card_messages);
                return (messages, recheck);
            }
            Square::CommunityChest => {
                let card = draw_card(&mut self.chest_deck, &mut self.rng, community_chest_cards);
                messages.push(format!("Community Chest card: {}", card.text));
                let (mut card_messages, recheck) = self.apply_card(player_index, card);
                messages.append(&mut card_messages);
                return (messages, recheck);
            }
            Square::GoToJail => {
                self.players[player_index].position = JAIL_POSITION;
                self.players[player_index].in_jail = true;
                self.players[player_index].jail_turns_left = MAX_JAIL_TURNS;
                messages.push("Go directly to jail.".to_string());
            }
        }
        (messages, false)
    }

    fn apply_card(&mut self, player_index: usize, card: Card) -> (Vec<String>, bool) {
        let mut messages = Vec::new();
        match card.effect {
            CardEffect::AdvanceTo(target) => {
                messages.extend(self.move_to(player_index, target, true));
                return (messages, true);
            }
            CardEffect::MoveBy(offset) => {
                messages.extend(self.move_by(player_index, offset, false));
                return (messages, true);
            }
            CardEffect::Earn(amount) => {
                self.players[player_index].cash += amount;
                messages.push(format!("Received ${amount}."));
            }
            CardEffect::Pay(amount) => {
                self.players[player_index].cash -= amount;
                self.free_parking_pot += amount;
                messages.push(format!("Paid ${amount}."));
                messages.push(format!("Free Parking pot is now ${}.", self.free_parking_pot));
                self.resolve_cash_change(player_index, None, &mut messages);
            }
            CardEffect::GoToJail => {
                self.players[player_index].position = JAIL_POSITION;
                self.players[player_index].in_jail = true;
                self.players[player_index].jail_turns_left = MAX_JAIL_TURNS;
                messages.push("Card sent you to jail.".to_string());
            }
            CardEffect::FreeParkingBonus(amount) => {
                self.players[player_index].cash += amount;
                messages.push(format!("Collected ${amount} from the card."));
            }
        }
        (messages, false)
    }

    fn pay_rent(&mut self, payer: usize, position: usize, rent: i32, messages: &mut Vec<String>) {
        let Some(owner) = self.lots[position].as_ref().and_then(|lot| lot.owner) else {
            messages.push("No owner found.".to_string());
            return;
        };
        self.players[payer].cash -= rent;
        self.players[owner].cash += rent;
        messages.push(format!("Paid ${rent} rent to {}.", self.players[owner].name));
        self.resolve_cash_change(payer, Some(owner), messages);
    }

    fn calculate_rent(&self, position: usize) -> i32 {
        let Some(square) = self.board.get(position) else {
            return 0;
        };
        let Some(lot) = self.lots.get(position).and_then(|l| l.as_ref()) else {
            return 0;
        };
        match square {
            Square::Property { kind: PropertyKind::Street { base_rent, .. }, .. } => {
                if lot.hotel {
                    base_rent * 6
                } else {
                    base_rent * (1 + i32::from(lot.houses))
                }
            }
            Square::Property { kind: PropertyKind::Airport { base_rent }, .. } => *base_rent,
            Square::Property { kind: PropertyKind::Utility { base_rent }, .. } => *base_rent,
            _ => 0,
        }
    }

    fn player_owns_full_set(&self, player_index: usize, position: usize) -> bool {
        let Some(Square::Property { kind: PropertyKind::Street { group, .. }, .. }) = self.board.get(position) else {
            return false;
        };

        self.board
            .iter()
            .enumerate()
            .filter_map(|(idx, square)| match square {
            Square::Property { kind: PropertyKind::Street { group: other_group, .. }, .. } if other_group == group => {
                    Some(idx)
                }
                _ => None,
            })
            .all(|idx| self.lots[idx].as_ref().and_then(|lot| lot.owner) == Some(player_index))
    }

    fn resolve_cash_change(&mut self, player_index: usize, creditor: Option<usize>, messages: &mut Vec<String>) {
        if !self.players[player_index].active {
            return;
        }
        if self.players[player_index].cash >= 0 {
            return;
        }
        let debt = -self.players[player_index].cash;
        self.players[player_index].cash = 0;
        messages.push(format!("{} is bankrupt for ${debt}.", self.players[player_index].name));

        match creditor {
            Some(owner) if owner != player_index && self.players[owner].active => {
                self.transfer_assets(player_index, Some(owner));
                messages.push(format!("Assets transferred to {}.", self.players[owner].name));
            }
            _ => {
                self.transfer_assets(player_index, None);
                messages.push("Assets returned to the bank.".to_string());
            }
        }

        self.players[player_index].active = false;
        self.players[player_index].in_jail = false;
        self.players[player_index].jail_turns_left = 0;
        self.pending_purchase = None;
        self.extra_turn_pending = false;

        if self.current_player == player_index {
            self.current_player = self.next_active_player_from(player_index);
        }
    }

    fn transfer_assets(&mut self, bankrupt: usize, creditor: Option<usize>) {
        for lot in self.lots.iter_mut().flatten() {
            if lot.owner == Some(bankrupt) {
                lot.owner = creditor;
                if creditor.is_none() {
                    lot.houses = 0;
                    lot.hotel = false;
                }
            }
        }
    }

    fn net_worth(&self, player_index: usize) -> i32 {
        if !self.players[player_index].active {
            return 0;
        }
        let mut total = self.players[player_index].cash;
        for (idx, lot) in self.lots.iter().enumerate() {
            if lot.as_ref().and_then(|l| l.owner) == Some(player_index) {
                if let Some(cost) = self.board[idx].purchase_cost() {
                    total += cost;
                }
                if let Some(lot_state) = lot.as_ref() {
                    total += i32::from(lot_state.houses) * self.house_cost_at(idx);
                    if lot_state.hotel {
                        total += self.house_cost_at(idx);
                    }
                }
            }
        }
        total
    }

    fn house_cost_at(&self, position: usize) -> i32 {
        match self.board.get(position) {
            Some(Square::Property { kind: PropertyKind::Street { house_cost, .. }, .. }) => *house_cost,
            _ => 0,
        }
    }
}

fn building_label(lot: &LotState) -> String {
    if lot.hotel {
        " [hotel]".to_string()
    } else if lot.houses > 0 {
        format!(" [{} houses]", lot.houses)
    } else {
        String::new()
    }
}

fn escape_json(input: &str) -> String {
    input
        .chars()
        .flat_map(|c| match c {
            '"' => "\\\"".chars().collect::<Vec<_>>(),
            '\\' => "\\\\".chars().collect::<Vec<_>>(),
            '\n' => "\\n".chars().collect::<Vec<_>>(),
            '\r' => "\\r".chars().collect::<Vec<_>>(),
            '\t' => "\\t".chars().collect::<Vec<_>>(),
            _ => vec![c],
        })
        .collect()
}

fn draw_card(deck: &mut Vec<Card>, rng: &mut SimpleRng, refill: fn() -> Vec<Card>) -> Card {
    if deck.is_empty() {
        *deck = refill();
    }
    let idx = rng.range(deck.len());
    deck.swap_remove(idx)
}

fn default_board() -> Vec<Square> {
    vec![
        Square::Go,
        Square::Property {
            name: "Flinders Lane",
            cost: 60,
            kind: PropertyKind::Street {
                group: ColorGroup::Brown,
                house_cost: 50,
                base_rent: 2,
            },
        },
        Square::CommunityChest,
        Square::Property {
            name: "Acland Street",
            cost: 60,
            kind: PropertyKind::Street {
                group: ColorGroup::Brown,
                house_cost: 50,
                base_rent: 4,
            },
        },
        Square::Tax { name: "Income Tax", amount: 200 },
        Square::Property {
            name: "Sydney Airport",
            cost: 200,
            kind: PropertyKind::Airport { base_rent: 25 },
        },
        Square::Property {
            name: "Bourke Street",
            cost: 100,
            kind: PropertyKind::Street {
                group: ColorGroup::LightBlue,
                house_cost: 50,
                base_rent: 6,
            },
        },
        Square::Chance,
        Square::Property {
            name: "Swanston Street",
            cost: 100,
            kind: PropertyKind::Street {
                group: ColorGroup::LightBlue,
                house_cost: 50,
                base_rent: 6,
            },
        },
        Square::Property {
            name: "Elizabeth Street",
            cost: 120,
            kind: PropertyKind::Street {
                group: ColorGroup::LightBlue,
                house_cost: 50,
                base_rent: 8,
            },
        },
        Square::Jail,
        Square::Property {
            name: "Lygon Street",
            cost: 140,
            kind: PropertyKind::Street {
                group: ColorGroup::Pink,
                house_cost: 100,
                base_rent: 10,
            },
        },
        Square::Property {
            name: "Sydney Power Grid",
            cost: 150,
            kind: PropertyKind::Utility { base_rent: 10 },
        },
        Square::Property {
            name: "Chapel Street",
            cost: 140,
            kind: PropertyKind::Street {
                group: ColorGroup::Pink,
                house_cost: 100,
                base_rent: 10,
            },
        },
        Square::Property {
            name: "Collins Street",
            cost: 160,
            kind: PropertyKind::Street {
                group: ColorGroup::Pink,
                house_cost: 100,
                base_rent: 12,
            },
        },
        Square::Property {
            name: "Melbourne Airport",
            cost: 200,
            kind: PropertyKind::Airport { base_rent: 25 },
        },
        Square::Property {
            name: "King Street",
            cost: 180,
            kind: PropertyKind::Street {
                group: ColorGroup::Orange,
                house_cost: 100,
                base_rent: 14,
            },
        },
        Square::CommunityChest,
        Square::Property {
            name: "George Street",
            cost: 180,
            kind: PropertyKind::Street {
                group: ColorGroup::Orange,
                house_cost: 100,
                base_rent: 14,
            },
        },
        Square::Property {
            name: "Brunswick Street",
            cost: 200,
            kind: PropertyKind::Street {
                group: ColorGroup::Orange,
                house_cost: 100,
                base_rent: 16,
            },
        },
        Square::FreeParking,
        Square::Property {
            name: "Oxford Street",
            cost: 220,
            kind: PropertyKind::Street {
                group: ColorGroup::Red,
                house_cost: 150,
                base_rent: 18,
            },
        },
        Square::Chance,
        Square::Property {
            name: "Dandenong Road",
            cost: 220,
            kind: PropertyKind::Street {
                group: ColorGroup::Red,
                house_cost: 150,
                base_rent: 18,
            },
        },
        Square::Property {
            name: "Anzac Parade",
            cost: 240,
            kind: PropertyKind::Street {
                group: ColorGroup::Red,
                house_cost: 150,
                base_rent: 20,
            },
        },
        Square::Property {
            name: "Brisbane Airport",
            cost: 200,
            kind: PropertyKind::Airport { base_rent: 25 },
        },
        Square::Property {
            name: "Macquarie Street",
            cost: 260,
            kind: PropertyKind::Street {
                group: ColorGroup::Yellow,
                house_cost: 150,
                base_rent: 22,
            },
        },
        Square::Property {
            name: "Esplanade",
            cost: 260,
            kind: PropertyKind::Street {
                group: ColorGroup::Yellow,
                house_cost: 150,
                base_rent: 22,
            },
        },
        Square::Property {
            name: "Water Board",
            cost: 150,
            kind: PropertyKind::Utility { base_rent: 10 },
        },
        Square::Property {
            name: "Sturt Street",
            cost: 280,
            kind: PropertyKind::Street {
                group: ColorGroup::Yellow,
                house_cost: 150,
                base_rent: 24,
            },
        },
        Square::GoToJail,
        Square::Property {
            name: "Pacific Highway",
            cost: 300,
            kind: PropertyKind::Street {
                group: ColorGroup::Green,
                house_cost: 200,
                base_rent: 26,
            },
        },
        Square::Property {
            name: "North Terrace",
            cost: 300,
            kind: PropertyKind::Street {
                group: ColorGroup::Green,
                house_cost: 200,
                base_rent: 26,
            },
        },
        Square::CommunityChest,
        Square::Property {
            name: "St Kilda Road",
            cost: 320,
            kind: PropertyKind::Street {
                group: ColorGroup::Green,
                house_cost: 200,
                base_rent: 28,
            },
        },
        Square::Property {
            name: "Perth Airport",
            cost: 320,
            kind: PropertyKind::Airport { base_rent: 25 },
        },
        Square::Chance,
        Square::Property {
            name: "Stirling Highway",
            cost: 350,
            kind: PropertyKind::Street {
                group: ColorGroup::DarkBlue,
                house_cost: 200,
                base_rent: 35,
            },
        },
        Square::Tax { name: "Luxury Tax", amount: 100 },
        Square::Property {
            name: "Adelaide Terrace",
            cost: 400,
            kind: PropertyKind::Street {
                group: ColorGroup::DarkBlue,
                house_cost: 200,
                base_rent: 50,
            },
        },
    ]
}

fn chance_cards() -> Vec<Card> {
    vec![
        Card { text: "Advance to GO", effect: CardEffect::AdvanceTo(0) },
        Card { text: "Go to Jail", effect: CardEffect::GoToJail },
        Card { text: "Advance to Anzac Parade", effect: CardEffect::AdvanceTo(24) },
        Card { text: "Advance to Lygon Street", effect: CardEffect::AdvanceTo(11) },
        Card { text: "Go back 3 spaces", effect: CardEffect::MoveBy(-3) },
        Card { text: "Bank pays you dividend of $50", effect: CardEffect::Earn(50) },
        Card { text: "Pay poor tax of $15", effect: CardEffect::Pay(15) },
        Card { text: "Free Parking bonus", effect: CardEffect::FreeParkingBonus(100) },
    ]
}

fn community_chest_cards() -> Vec<Card> {
    vec![
        Card { text: "Advance to GO", effect: CardEffect::AdvanceTo(0) },
        Card { text: "Bank error in your favor", effect: CardEffect::Earn(200) },
        Card { text: "Doctor's fees", effect: CardEffect::Pay(50) },
        Card { text: "Go to Jail", effect: CardEffect::GoToJail },
        Card { text: "Holiday fund matures", effect: CardEffect::Earn(100) },
        Card { text: "Income tax refund", effect: CardEffect::Earn(20) },
        Card { text: "Free Parking bonus", effect: CardEffect::FreeParkingBonus(50) },
        Card { text: "Go back 3 spaces", effect: CardEffect::MoveBy(-3) },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game(players: usize) -> GameState {
        GameState::new(players, 12345)
    }

    fn set_owner(game: &mut GameState, position: usize, owner: usize) {
        game.lots[position].as_mut().expect("buyable square").owner = Some(owner);
    }

    #[test]
    fn dice_roll_helpers_work() {
        let roll = DiceRoll { die1: 3, die2: 3 };
        assert_eq!(roll.total(), 6);
        assert!(roll.is_double());

        let roll = DiceRoll { die1: 2, die2: 5 };
        assert_eq!(roll.total(), 7);
        assert!(!roll.is_double());
    }

    #[test]
    fn rng_is_deterministic_for_same_seed() {
        let mut a = SimpleRng::seeded(99);
        let mut b = SimpleRng::seeded(99);

        for _ in 0..32 {
            assert_eq!(a.roll_die(), b.roll_die());
            assert_eq!(a.range(17), b.range(17));
        }

        assert_eq!(SimpleRng::seeded(1).range(0), 0);
    }

    #[test]
    fn board_has_all_40_expected_positions() {
        let game = game(2);

        assert_eq!(game.board.len(), 40);

        let expected: Vec<(usize, &'static str, &'static str)> = vec![
            (0, "GO", ""),
            (1, "Flinders Lane", "Brown"),
            (2, "Community Chest", ""),
            (3, "Acland Street", "Brown"),
            (4, "Income Tax", ""),
            (5, "Sydney Airport", "Airport"),
            (6, "Bourke Street", "Light Blue"),
            (7, "Chance", ""),
            (8, "Swanston Street", "Light Blue"),
            (9, "Elizabeth Street", "Light Blue"),
            (10, "Jail", ""),
            (11, "Lygon Street", "Pink"),
            (12, "Sydney Power Grid", "Utility"),
            (13, "Chapel Street", "Pink"),
            (14, "Collins Street", "Pink"),
            (15, "Melbourne Airport", "Airport"),
            (16, "King Street", "Orange"),
            (17, "Community Chest", ""),
            (18, "George Street", "Orange"),
            (19, "Brunswick Street", "Orange"),
            (20, "Free Parking", ""),
            (21, "Oxford Street", "Red"),
            (22, "Chance", ""),
            (23, "Dandenong Road", "Red"),
            (24, "Anzac Parade", "Red"),
            (25, "Brisbane Airport", "Airport"),
            (26, "Macquarie Street", "Yellow"),
            (27, "Esplanade", "Yellow"),
            (28, "Water Board", "Utility"),
            (29, "Sturt Street", "Yellow"),
            (30, "Go To Jail", ""),
            (31, "Pacific Highway", "Green"),
            (32, "North Terrace", "Green"),
            (33, "Community Chest", ""),
            (34, "St Kilda Road", "Green"),
            (35, "Perth Airport", "Airport"),
            (36, "Chance", ""),
            (37, "Stirling Highway", "Dark Blue"),
            (38, "Luxury Tax", ""),
            (39, "Adelaide Terrace", "Dark Blue"),
        ];

        for (index, expected_name, expected_label) in expected {
            let square = &game.board[index];
            assert_eq!(square.name(), expected_name, "unexpected square name at index {index}");
            assert_eq!(square.board_label(), expected_label, "unexpected label at index {index}");
        }

        assert!(matches!(game.board[0], Square::Go));
        assert!(matches!(game.board[2], Square::CommunityChest));
        assert!(matches!(game.board[7], Square::Chance));
        assert!(matches!(game.board[10], Square::Jail));
        assert!(matches!(game.board[20], Square::FreeParking));
        assert!(matches!(game.board[30], Square::GoToJail));
    }

    #[test]
    fn street_colour_groups_are_complete() {
        let game = game(2);

        let mut brown = 0;
        let mut light_blue = 0;
        let mut pink = 0;
        let mut orange = 0;
        let mut red = 0;
        let mut yellow = 0;
        let mut green = 0;
        let mut dark_blue = 0;

        for square in &game.board {
            if let Square::Property { kind: PropertyKind::Street { group, .. }, .. } = square {
                match group {
                    ColorGroup::Brown => brown += 1,
                    ColorGroup::LightBlue => light_blue += 1,
                    ColorGroup::Pink => pink += 1,
                    ColorGroup::Orange => orange += 1,
                    ColorGroup::Red => red += 1,
                    ColorGroup::Yellow => yellow += 1,
                    ColorGroup::Green => green += 1,
                    ColorGroup::DarkBlue => dark_blue += 1,
                }
            }
        }

        assert_eq!(brown, 2);
        assert_eq!(light_blue, 3);
        assert_eq!(pink, 3);
        assert_eq!(orange, 3);
        assert_eq!(red, 3);
        assert_eq!(yellow, 3);
        assert_eq!(green, 3);
        assert_eq!(dark_blue, 2);
    }

    #[test]
    fn airports_are_all_present() {
        let game = game(2);
        let airports: Vec<_> = game
            .board
            .iter()
            .enumerate()
            .filter(|(_, square)| matches!(square, Square::Property { kind: PropertyKind::Airport { .. }, .. }))
            .map(|(idx, square)| (idx, square.name()))
            .collect();

        assert_eq!(airports, vec![(5, "Sydney Airport"), (15, "Melbourne Airport"), (25, "Brisbane Airport"), (35, "Perth Airport")]);
    }

    #[test]
    fn movement_wraps_from_last_square_to_go_and_collects_pass_go_cash() {
        let mut game = game(2);
        game.players[0].position = 39;
        let messages = game.move_by(0, 1, true);

        assert_eq!(game.players[0].position, 0);
        assert_eq!(game.players[0].cash, START_CASH + PASS_GO_CASH);
        assert!(messages.iter().any(|m| m.contains("Passed GO and collected $200.")));
    }

    #[test]
    fn move_to_from_late_board_space_collects_pass_go_cash_when_wrapping() {
        let mut game = game(2);
        game.players[0].position = 39;
        let messages = game.move_to(0, 3, true);

        assert_eq!(game.players[0].position, 3);
        assert_eq!(game.players[0].cash, START_CASH + PASS_GO_CASH);
        assert!(messages.iter().any(|m| m.contains("Passed GO and collected $200.")));
    }

    #[test]
    fn game_initialises_with_expected_board_size_and_lots() {
        let game = game(2);
        assert_eq!(game.board.len(), BOARD_SIZE);
        assert_eq!(game.lots.len(), BOARD_SIZE);
        assert_eq!(game.players.len(), 2);
        assert_eq!(game.players[0].cash, START_CASH);
    }

    #[test]
    fn legal_actions_change_when_purchase_is_pending() {
        let mut game = game(2);
        assert_eq!(game.legal_actions(), vec![Action::Roll, Action::Trade(TradeOffer {
            from: 0,
            to: 0,
            cash_from_offer: 0,
            cash_to_offer: 0,
            give_positions: vec![],
            receive_positions: vec![],
        })]);

        game.pending_purchase = Some(PendingPurchase { position: 1 });
        assert_eq!(game.legal_actions(), vec![Action::BuyProperty, Action::DeclineProperty]);
    }

    #[test]
    fn buying_property_deducts_cash_and_assigns_owner() {
        let mut game = game(2);
        game.players[0].position = 1;

        let outcome = game.resolve_position(0);
        assert!(outcome.iter().any(|m| m.contains("Buy for $60 or decline.")));
        assert_eq!(game.pending_purchase_square().map(Square::name), Some("Flinders Lane"));

        let result = game.step(Action::BuyProperty);
        assert!(result.messages.iter().any(|m| m.contains("Bought Flinders Lane for $60.")));
        assert_eq!(game.lots[1].as_ref().and_then(|l| l.owner), Some(0));
        assert_eq!(game.players[0].cash, START_CASH - 60);
        assert!(game.pending_purchase.is_none());
    }

    #[test]
    fn declining_property_clears_pending_purchase_without_sale() {
        let mut game = game(2);
        game.players[0].position = 3;
        let _ = game.resolve_position(0);
        assert!(game.pending_purchase.is_some());

        let result = game.step(Action::DeclineProperty);
        assert!(result.messages.iter().any(|m| m.contains("Declined purchase of Acland Street.")));
        assert!(game.pending_purchase.is_none());
        assert_eq!(game.lots[3].as_ref().and_then(|l| l.owner), None);
    }

    #[test]
    fn jail_roll_with_doubles_releases_player_and_grants_extra_turn() {
        let mut game = game(2);
        game.players[0].in_jail = true;
        game.players[0].jail_turns_left = 3;
        game.players[0].position = JAIL_POSITION;

        let mut messages = Vec::new();
        game.handle_jail_roll(0, DiceRoll { die1: 4, die2: 4 }, &mut messages);

        assert!(messages.iter().any(|m| m.contains("Rolled doubles and got out of jail.")));
        assert!(game.players[0].active);
        assert!(!game.players[0].in_jail);
        assert_eq!(game.players[0].jail_turns_left, 0);
        assert_eq!(game.players[0].position, JAIL_POSITION + 8);
        assert!(game.extra_turn_pending);
    }

    #[test]
    fn jail_roll_without_doubles_counts_down_and_releases_on_last_turn() {
        let mut game = game(2);
        game.players[0].in_jail = true;
        game.players[0].jail_turns_left = 2;
        game.players[0].position = JAIL_POSITION;

        let mut messages = Vec::new();
        game.handle_jail_roll(0, DiceRoll { die1: 2, die2: 3 }, &mut messages);

        assert!(messages.iter().any(|m| m.contains("No doubles. Jail turns left: 1.")));
        assert!(game.players[0].in_jail);
        assert_eq!(game.players[0].jail_turns_left, 1);
        assert_eq!(game.players[0].position, JAIL_POSITION);

        let mut messages = Vec::new();
        game.handle_jail_roll(0, DiceRoll { die1: 1, die2: 2 }, &mut messages);

        assert!(messages.iter().any(|m| m.contains("Paid $50 to get out")));
        assert!(!game.players[0].in_jail);
        assert_eq!(game.players[0].jail_turns_left, 0);
        assert_eq!(game.players[0].cash, START_CASH - JAIL_FINE);
        assert_eq!(game.players[0].position, JAIL_POSITION + 3);
    }

    #[test]
    fn rent_payment_transfers_cash_and_bankrupts_player_with_assets() {
        let mut game = game(2);
        set_owner(&mut game, 1, 0);
        set_owner(&mut game, 3, 1);

        game.players[1].cash = 0;
        game.players[1].position = 1;

        let messages = game.resolve_position(1);

        assert!(messages.iter().any(|m| m.contains("Paid $2 rent to Player 1.")));
        assert!(messages.iter().any(|m| m.contains("Player 2 is bankrupt for $2.")));
        assert!(!game.players[1].active);
        assert_eq!(game.players[1].cash, 0);
        assert_eq!(game.lots[3].as_ref().and_then(|l| l.owner), Some(0));
        assert_eq!(game.lots[1].as_ref().and_then(|l| l.owner), Some(0));
    }

    #[test]
    fn houses_require_full_set_and_hotel_requires_four_houses() {
        let mut game = game(2);
        set_owner(&mut game, 1, 0);
        set_owner(&mut game, 3, 0);

        let mut messages = Vec::new();
        game.handle_build_house(0, 1, &mut messages);
        assert!(messages.iter().any(|m| m.contains("Built a house on Flinders Lane for $50.")));
        assert_eq!(game.lots[1].as_ref().unwrap().houses, 1);
        assert_eq!(game.players[0].cash, START_CASH - 50);

        game.handle_build_house(0, 1, &mut messages);
        game.handle_build_house(0, 1, &mut messages);
        game.handle_build_house(0, 1, &mut messages);
        assert_eq!(game.lots[1].as_ref().unwrap().houses, 4);

        game.handle_build_hotel(0, 1, &mut messages);
        assert!(messages.iter().any(|m| m.contains("Built a hotel on Flinders Lane for $50.")));
        assert!(game.lots[1].as_ref().unwrap().hotel);
        assert_eq!(game.lots[1].as_ref().unwrap().houses, 0);
        assert_eq!(game.players[0].cash, START_CASH - 250);
    }

    #[test]
    fn cannot_build_before_complete_set_or_without_four_houses() {
        let mut game = game(2);
        set_owner(&mut game, 1, 0);

        let mut messages = Vec::new();
        game.handle_build_house(0, 1, &mut messages);
        assert!(messages.iter().any(|m| m.contains("does not own the full set")));
        assert_eq!(game.lots[1].as_ref().unwrap().houses, 0);

        set_owner(&mut game, 3, 0);
        game.handle_build_hotel(0, 1, &mut messages);
        assert!(messages.iter().any(|m| m.contains("Build 4 houses before building a hotel.")));
        assert!(!game.lots[1].as_ref().unwrap().hotel);
    }
}
