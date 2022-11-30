use hattrick_packets_lib::clientinfo::ClientInfo;
use hattrick_packets_lib::gamestate::GameState;
use hattrick_packets_lib::gametypes::GameType;
use hattrick_packets_lib::pong::PongClientState;
use hattrick_packets_lib::team::Team;

pub fn get_pong_state_for_ai(
    team_id: &Team,
    local_gs: &GameState,
    client_packet: &mut ClientInfo,
    previous_pcs: &PongClientState,
) -> PongClientState {
    // paddle heights based on team
    // BlueTeam => 10.0,
    // RedTeam => 550.0,
    {
        let paddle_y = {
            match &team_id {
                Team::RedTeam => 550.0,
                Team::BlueTeam => 10.0,
            }
        };

        let ball_height = match &local_gs.game_type {
            GameType::PONG(pgs) => pgs.ball_y,
            _ => -1.0, // only get a ball height if we are playing pong
        };

        
        PongClientState {
            paddle_x: {
                // distance at which the ai stops handling the ball perfectly, pretty much a difficulty modifier?
                if (ball_height - paddle_y).abs() < 100.0 {
                    previous_pcs.paddle_x
                } else {
                    client_packet.mouse_pos.0
                }
            },
            paddle_y,
        }
    }
}
