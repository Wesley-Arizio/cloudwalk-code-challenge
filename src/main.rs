use std::collections::HashMap;

struct Game {
    pub total_kills: u32,
    pub players: Vec<String>,
    pub kills: HashMap<String, u32>,
}

impl Game {
    pub fn new() -> Self {
        Self {
            total_kills: 0,
            players: vec![],
            kills: HashMap::new(),
        }
    }

    pub fn increase_total_kills(&mut self) {
        self.total_kills += 1;
    }

    pub fn decrease_total_kills(&mut self) {
        self.total_kills -= 1;
    }

    pub fn add_player(&mut self, player_name: &str) {
        self.players.push(player_name.to_string())
    }

    pub fn increase_player_kills(&mut self, player_name: &str) {
        *self.kills.entry(player_name.to_string()).or_default() += 1;
    }

    pub fn decrease_player_kills(&mut self, player_name: &str) {
        self.kills
            .entry(player_name.to_string())
            .and_modify(|e| *e -= 1);
    }
}

fn main() -> std::io::Result<()> {
    let game = Game::new();
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_game_structure() {
        // Test default values for a new instance
        let mut game = Game::new();
        assert_eq!(game.total_kills, 0);
        assert_eq!(game.players.len(), 0);
        assert_eq!(game.kills.len(), 0);

        game.increase_total_kills();
        game.increase_total_kills();
        assert_eq!(game.total_kills, 2);

        game.decrease_total_kills();
        assert_eq!(game.total_kills, 1);

        game.add_player("john doe");
        game.add_player("joana doe");
        assert_eq!(game.players.len(), 2);
        assert_eq!(game.players[0], "john doe");
        assert_eq!(game.players[1], "joana doe");

        // Increase players kills by demand
        game.increase_player_kills("john doe");
        assert_eq!(*game.kills.get("john doe").unwrap(), 1u32);
        game.increase_player_kills("john doe");
        assert_eq!(*game.kills.get("john doe").unwrap(), 2u32);
        game.increase_player_kills("john doe");
        assert_eq!(*game.kills.get("john doe").unwrap(), 3u32);

        // Decrease players kills by demand
        game.decrease_player_kills("john doe");
        assert_eq!(*game.kills.get("john doe").unwrap(), 2u32);
        game.decrease_player_kills("john doe");
        assert_eq!(*game.kills.get("john doe").unwrap(), 1u32);
    }
}
