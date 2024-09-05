mod quiz_game;
use quiz_game::quiz_game::play_trivia_game;
#[tokio::main]
async fn main() {
    play_trivia_game().await;
}
