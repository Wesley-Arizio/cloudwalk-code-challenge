use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};

#[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
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

    fn handle_line(
        line: &str,
        current_game: &mut GameMatch,
        games: &mut Vec<GameMatch>,
        index: usize,
    ) -> Result<(), String> {
        if line.contains("InitGame:") && index > 2 {
            games.push(current_game.clone());
            *current_game = GameMatch::new();
            return Ok(());
        };

        if !line.contains("Kill") || line.contains("---") {
            return Ok(());
        }

        let rest = line
            .split("killed")
            .map(|i| i.trim())
            .collect::<Vec<&str>>();

        let invalid_killer_name = rest[0].ends_with(":");
        let invalid_player_killed_name = rest[1].trim().starts_with("by");

        if invalid_killer_name || invalid_player_killed_name {
            eprintln!("killer or killed player's name is invalid");
            return Ok(());
        }

        current_game.increase_total_kills();

        let killer = rest[0]
            .split(":")
            .last()
            // Impossible case since we validated that there is something after ':'
            .ok_or_else(|| "No killer found".to_string())?
            .trim();

        // In case that there is no cause of death
        let killed = rest[1].split("by").collect::<Vec<&str>>();
        if killed.len() < 2 {
            return Err(
                "invalid format: there is no information about player killed or the cause of death"
                    .to_string(),
            );
        }

        let player_killed = killed[0].trim();
        let mean = killed[1].trim();

        current_game.add_player(player_killed);
        current_game.increase_kill_by_mean(MeansOfDeath::try_from(mean)?);

        if !killer.contains("<world>") {
            current_game.add_player(killer);
            current_game.increase_player_kills(killer)
        } else {
            current_game.decrease_player_kills(player_killed);
        }
        Ok(())
    }

    pub fn generate_report(&self) -> Result<(), String> {
        let file = File::open(&self.filename).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);
        let mut games: Vec<GameMatch> = vec![];
        let mut current_game = GameMatch::new();

        for (i, line) in reader.lines().enumerate() {
            let line = line.map_err(|e| e.to_string())?;

            match Game::handle_line(&line, &mut current_game, &mut games, i) {
                Ok(_) => continue,
                Err(e) => return Err(e),
            }
        }

        // Add last game to the history even it it wasn't finshed yet (in case the log file is over);
        games.push(current_game);

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
    fn test_game_match_structure() {
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

    #[test]
    fn test_handle_line() {
        let mut games: Vec<GameMatch> = vec![];
        let mut current_game = GameMatch::new();

        // line does not contem Kill and not dashes
        assert!(Game::handle_line(r#"20:34 ClientUserinfoChanged: 2 n\Isgalamido\t\0\model\xian/default\hmodel\xian/default\g_redteam\\g_blueteam\\c1\4\c2\5\hc\100\w\0\l\0\tt\0\tl\0"#, &mut current_game, &mut games, 1).is_ok());
        assert_eq!(games.len(), 0);
        assert_eq!(current_game.total_kills, 0);

        // line contain dashes
        assert!(Game::handle_line(
            r#"  0:00 ------------------------------------------------------------"#,
            &mut current_game,
            &mut games,
            1
        )
        .is_ok());
        assert_eq!(games.len(), 0);
        assert_eq!(current_game.total_kills, 0);

        // Line contain InitGame and index is less than 2
        assert!(Game::handle_line(r#"  0:00 InitGame: \sv_floodProtect\1\sv_maxPing\0\sv_minPing\0\sv_maxRate\10000\sv_minRate\0\sv_hostname\Code Miner Server\g_gametype\0\sv_privateClients\2\sv_maxclients\16\sv_allowDownload\0\dmflags\0\fraglimit\20\timelimit\15\g_maxGameClients\0\capturelimit\8\version\ioq3 1.36 linux-x86_64 Apr 12 2009\protocol\68\mapname\q3dm17\gamename\baseq3\g_needpass\0"#, &mut current_game, &mut games, 1).is_ok());
        assert_eq!(games.len(), 0);
        assert_eq!(current_game.total_kills, 0);

        // Line contain InitGame in the middle of the file
        assert!(Game::handle_line(r#"  0:00 InitGame: \sv_floodProtect\1\sv_maxPing\0\sv_minPing\0\sv_maxRate\10000\sv_minRate\0\sv_hostname\Code Miner Server\g_gametype\0\sv_privateClients\2\sv_maxclients\16\sv_allowDownload\0\dmflags\0\fraglimit\20\timelimit\15\g_maxGameClients\0\capturelimit\8\version\ioq3 1.36 linux-x86_64 Apr 12 2009\protocol\68\mapname\q3dm17\gamename\baseq3\g_needpass\0"#, &mut current_game, &mut games, 345).is_ok());
        assert_eq!(games.len(), 1);
        assert_eq!(current_game.total_kills, 0);

        // Line does not contain the name of the killer
        assert!(Game::handle_line(
            r#" 20:54 Kill: 1022 2 22:  killed Isgalamido by MOD_TRIGGER_HURT"#,
            &mut current_game,
            &mut games,
            4
        )
        .is_ok());
        // Creates a new game match but does not finish this line of report
        assert_eq!(games.len(), 1);
        assert_eq!(current_game.total_kills, 0);

        // Line does not contain the name of the player killed
        assert!(Game::handle_line(
            r#" 20:54 Kill: 1022 2 22: <world> killed  by MOD_TRIGGER_HURT"#,
            &mut current_game,
            &mut games,
            5
        )
        .is_ok(),);
        assert_eq!(games.len(), 1);
        assert_eq!(current_game.total_kills, 0);

        // Valid line, world kills a player
        assert!(Game::handle_line(
            r#" 20:54 Kill: 1022 2 22: <world> killed Isgalamido by MOD_TRIGGER_HURT"#,
            &mut current_game,
            &mut games,
            6
        )
        .is_ok());
        assert_eq!(games.len(), 1);
        assert_eq!(current_game.total_kills, 1);
        assert_eq!(current_game.players.len(), 1);
        assert!(current_game.players.contains(&"Isgalamido".to_string()));
        assert_eq!(current_game.kills.len(), 0);
        assert_eq!(
            current_game
                .kills_by_means
                .get(&MeansOfDeath::ModTriggerHurt)
                .unwrap()
                .clone(),
            1
        );

        // Valid line, Isgalamido kills a player
        assert!(Game::handle_line(
            r#" 20:54 Kill: 1022 2 22: Isgalamido killed Dono da Bola by MOD_TRIGGER_HURT"#,
            &mut current_game,
            &mut games,
            7
        )
        .is_ok());
        assert_eq!(games.len(), 1);
        assert_eq!(current_game.total_kills, 2);
        assert_eq!(current_game.players.len(), 2);
        assert!(current_game.players.contains(&"Isgalamido".to_string()));
        assert!(current_game.players.contains(&"Dono da Bola".to_string()));
        assert_eq!(current_game.kills.len(), 1);
        assert_eq!(current_game.kills.get("Isgalamido").unwrap().clone(), 1);
        assert_eq!(
            current_game
                .kills_by_means
                .get(&MeansOfDeath::ModTriggerHurt)
                .unwrap()
                .clone(),
            2
        );

        // Valid line, Isgalamido is killed by world and loses 1 kill
        assert!(Game::handle_line(
            r#" 20:54 Kill: 1022 2 22: <world> killed Isgalamido by MOD_FALLING"#,
            &mut current_game,
            &mut games,
            8
        )
        .is_ok());
        assert_eq!(games.len(), 1);
        assert_eq!(current_game.total_kills, 3);
        assert_eq!(current_game.kills.get("Isgalamido").unwrap().clone(), 0);
        assert_eq!(
            current_game
                .kills_by_means
                .get(&MeansOfDeath::ModFalling)
                .unwrap()
                .clone(),
            1
        );

        let previous_game_match = current_game.clone();

        // Initialize another game match
        assert!(Game::handle_line(r#"  0:00 InitGame: \sv_floodProtect\1\sv_maxPing\0\sv_minPing\0\sv_maxRate\10000\sv_minRate\0\sv_hostname\Code Miner Server\g_gametype\0\sv_privateClients\2\sv_maxclients\16\sv_allowDownload\0\dmflags\0\fraglimit\20\timelimit\15\g_maxGameClients\0\capturelimit\8\version\ioq3 1.36 linux-x86_64 Apr 12 2009\protocol\68\mapname\q3dm17\gamename\baseq3\g_needpass\0"#, &mut current_game, &mut games, 385).is_ok());
        assert_eq!(games.len(), 2);
        assert_eq!(games[1], previous_game_match);
    }
}
