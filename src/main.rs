use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};

#[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
enum MeansOfDeath {
    ModUnknown,
    ModShotgun,
    ModGauntlet,
    ModMachinegun,
    ModGrenade,
    ModGrenadeSplash,
    ModRocket,
    ModRocketSplash,
    ModPlasma,
    ModPlasmaSplash,
    ModRailgun,
    ModLightning,
    ModBfg,
    ModBfgSplash,
    ModWater,
    ModSlime,
    ModLava,
    ModCrush,
    ModTelefrag,
    ModFalling,
    ModSuicide,
    ModTargetLaser,
    ModTriggerHurt,
    ModNail,
    ModChaingun,
    ModProximityMine,
    ModKamikaze,
    ModJuiced,
    ModGrapple,
}

impl TryFrom<&str> for MeansOfDeath {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "MOD_UNKNOWN" => Ok(Self::ModUnknown),
            "MOD_SHOTGUN" => Ok(Self::ModShotgun),
            "MOD_GAUNTLET" => Ok(Self::ModGauntlet),
            "MOD_GRENADE" => Ok(Self::ModGrenade),
            "MOD_GRENADE_SPLASH" => Ok(Self::ModGrenadeSplash),
            "MOD_ROCKET" => Ok(Self::ModRocket),
            "MOD_ROCKET_SPLASH" => Ok(Self::ModRocketSplash),
            "MOD_PLASMA" => Ok(Self::ModPlasma),
            "MOD_PLASMA_SPLASH" => Ok(Self::ModPlasmaSplash),
            "MOD_RAILGUN" => Ok(Self::ModRailgun),
            "MOD_LIGHTNING" => Ok(Self::ModLightning),
            "MOD_BFG" => Ok(Self::ModBfg),
            "MOD_BFG_SPLASH" => Ok(Self::ModBfgSplash),
            "MOD_WATER" => Ok(Self::ModWater),
            "MOD_SLIME" => Ok(Self::ModSlime),
            "MOD_LAVA" => Ok(Self::ModLava),
            "MOD_CRUSH" => Ok(Self::ModCrush),
            "MOD_TELEFRAG" => Ok(Self::ModTelefrag),
            "MOD_FALLING" => Ok(Self::ModFalling),
            "MOD_SUICIDE" => Ok(Self::ModSuicide),
            "MOD_TARGET_LASER" => Ok(Self::ModTargetLaser),
            "MOD_TRIGGER_HURT" => Ok(Self::ModTriggerHurt),
            "MOD_NAIL" => Ok(Self::ModNail),
            "MOD_CHAINGUN" => Ok(Self::ModChaingun),
            "MOD_MACHINEGUN" => Ok(Self::ModMachinegun),
            "MOD_PROXIMITY_MINE" => Ok(Self::ModProximityMine),
            "MOD_KAMIKAZE" => Ok(Self::ModKamikaze),
            "MOD_JUICED" => Ok(Self::ModJuiced),
            "MOD_GRAPPLE" => Ok(Self::ModGrapple),
            _ => Err(format!("Invalid mean of death: '{}'", value)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct GameMatch {
    pub total_kills: u32,
    pub players: Vec<String>,
    pub kills: HashMap<String, u32>,
    pub kills_by_means: HashMap<MeansOfDeath, u32>,
}

impl GameMatch {
    pub fn new() -> Self {
        Self {
            total_kills: 0,
            players: vec![],
            kills: HashMap::new(),
            kills_by_means: HashMap::new(),
        }
    }

    pub fn increase_total_kills(&mut self) {
        self.total_kills += 1;
    }

    pub fn add_player(&mut self, player_name: &str) {
        if !self.players.contains(&player_name.to_string()) {
            self.players.push(player_name.to_string());
        }
    }

    pub fn increase_player_kills(&mut self, player_name: &str) {
        *self.kills.entry(player_name.to_string()).or_default() += 1;
    }

    pub fn decrease_player_kills(&mut self, player_name: &str) {
        self.kills.entry(player_name.to_string()).and_modify(|e| {
            if *e > u32::MIN {
                *e -= 1
            }
        });
    }

    pub fn increase_kill_by_mean(&mut self, mean: MeansOfDeath) {
        *self.kills_by_means.entry(mean).or_default() += 1;
    }
}

struct Game {
    filename: String,
}

impl Game {
    pub fn new(filename: &str) -> Self {
        Self {
            filename: filename.to_string(),
        }
    }

    pub fn generate_report(&self) -> Result<(), String> {
        let file = File::open(&self.filename).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);
        let mut games: Vec<GameMatch> = vec![];
        let mut game = GameMatch::new();

        for (i, line) in reader.lines().enumerate() {
            let line = line.map_err(|e| e.to_string())?;

            // Verify if the new game is in the init of the file (avoid adding an empty game match to the history)
            if line.contains("InitGame:") && i > 2 {
                games.push(game);
                game = GameMatch::new();
                continue;
            };

            if !line.contains("Kill") || line.contains("---") {
                continue;
            }

            let rest = line.split("killed").collect::<Vec<&str>>();
            if rest.len() < 2 {
                return Err(
                    "invalid format: there is no information about the killer or the player killed"
                        .to_string(),
                );
            };

            game.increase_total_kills();

            let killer = rest[0]
                .split(":")
                .last()
                .ok_or_else(|| "No killer found".to_string())?
                .trim();

            let killed = rest[1].split("by").collect::<Vec<&str>>();
            if killed.len() < 2 {
                return Err(
                    "invalid format: there is no information about player killed or the deaths mean"
                        .to_string(),
                );
            }

            let player_killed = killed[0].trim();
            let mean = killed[1].trim();

            game.add_player(player_killed);
            game.increase_kill_by_mean(MeansOfDeath::try_from(mean)?);

            if !killer.contains("<world>") {
                game.add_player(killer);
                game.increase_player_kills(killer)
            } else {
                game.decrease_player_kills(player_killed);
            }
        }

        // Add last game to the history even it it wasn't finshed yet (in case the log file is over);
        games.push(game);

        Game::write_file(games)?;
        Ok(())
    }

    fn write_file(data: Vec<GameMatch>) -> Result<(), String> {
        let file = File::create("output.json").map_err(|e| e.to_string())?;
        let mut writer = BufWriter::new(file);
        serde_json::to_writer(&mut writer, &data).map_err(|e| e.to_string())?;
        writer.flush().map_err(|e| e.to_string())?;

        Ok(())
    }
}

fn main() -> Result<(), String> {
    let game = Game::new("qgames.log");

    game.generate_report()?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_game_structure() {
        // Test default values for a new instance
        let mut game = GameMatch::new();
        assert_eq!(game.total_kills, 0);
        assert_eq!(game.players.len(), 0);
        assert_eq!(game.kills.len(), 0);

        game.increase_total_kills();
        game.increase_total_kills();
        assert_eq!(game.total_kills, 2);

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

        assert_eq!(game.kills_by_means.len(), 0);
        game.increase_kill_by_mean(MeansOfDeath::ModBfg);
        assert_eq!(
            *game.kills_by_means.get(&MeansOfDeath::ModBfg).unwrap(),
            1u32
        );
        game.increase_kill_by_mean(MeansOfDeath::ModBfg);
        assert_eq!(
            *game.kills_by_means.get(&MeansOfDeath::ModBfg).unwrap(),
            2u32
        );
    }
}
