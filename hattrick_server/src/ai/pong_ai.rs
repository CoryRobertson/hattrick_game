use hattrick_packets_lib::clientinfo::ClientInfo;
use hattrick_packets_lib::gamestate::GameState;
use hattrick_packets_lib::gametypes::GameType;
use hattrick_packets_lib::pong::{
    get_pong_paddle_width, PongClientState, BLUE_TEAM_PADDLE_Y, PADDLE_MOVE_SPEED,
    PONG_PADDLE_WIDTH, RED_TEAM_PADDLE_Y,
};
use hattrick_packets_lib::team::Team;
use hattrick_packets_lib::{Magnitude, GAME_WIDTH};

/// Multiplicative modifier applied to the ball speed to determine how close the ball gets to the paddle before the AI stops reacting.
static REACTION_DISTANCE_MODIFIER: f32 = 2.0;
static PADDLE_MOVE_SPEED_MODIFIER: f32 = 3.0;

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
        let reaction_distance: f32 = {
            let ball_speed: f32 = match &local_gs.game_type {
                GameType::PONG(pgs) => {
                    let vec = (pgs.ball_xvel, pgs.ball_yvel);
                    vec.mag()
                }
                _ => 0.0,
            };
            REACTION_DISTANCE_MODIFIER * ball_speed
        }; // reaction distance is the distance at which the pong ai will stop moving from the ball, useful for scaling difficulty.

        let paddle_y = {
            match &team_id {
                Team::RedTeam => RED_TEAM_PADDLE_Y,
                Team::BlueTeam => BLUE_TEAM_PADDLE_Y,
            }
        }; // paddle height of the given ai

        let paddle_width =
            get_pong_paddle_width(&local_gs.client_list, team_id).clamp(0.0, PONG_PADDLE_WIDTH);

        let ball_height = match &local_gs.game_type {
            GameType::PONG(pgs) => pgs.ball_y,
            _ => -1.0, // only get a ball height if we are playing pong
        }; // ball y value

        // TODO: make the ai paddle go to the location the ball will eventually be aka its position plus the balls magnitude.

        let new_paddle_x = {
            if (ball_height - paddle_y).abs() < reaction_distance
                || ball_height > RED_TEAM_PADDLE_Y
                || ball_height < BLUE_TEAM_PADDLE_Y
            {
                // if the ball distance to the paddle is less than the reaction distance, OR the ball is above the top paddle OR below the bottom paddle.
                previous_pcs.paddle_x // just hold the paddle position.
            } else {
                let previous_client_x = previous_pcs.paddle_x;
                // client_packet.mouse_pos.0 - (paddle_width / 2.0) // center paddle to ball
                if (client_packet.mouse_pos.0 - (paddle_width / 2.0)) < previous_client_x {
                    // mouse is to the left of the paddle at the moment
                    previous_client_x - (PADDLE_MOVE_SPEED * PADDLE_MOVE_SPEED_MODIFIER)
                } else {
                    // mouse is to the right of the paddle at the moment
                    previous_client_x + (PADDLE_MOVE_SPEED * PADDLE_MOVE_SPEED_MODIFIER)
                }
            }
        };

        PongClientState {
            paddle_x: {
                // distance at which the ai stops handling the ball perfectly, pretty much a difficulty modifier?
                new_paddle_x.clamp(0.0, GAME_WIDTH - paddle_width)
            },
            paddle_y,
            time_of_power_hit: previous_pcs.time_of_power_hit,
        }
    }
}
