use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::time::{Duration, SystemTime};

pub fn gen_game_room_code() -> String {
    let mut rng = thread_rng();
    let output: String = std::iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .map(char::from)
        .filter(|x| x.is_alphabetic())
        .take(4)
        .collect();
    output.to_uppercase()
}

pub fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub fn unix_timestamp_to(to: Duration) -> u64 {
    let then = SystemTime::now() + to;
    then.duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values() {
        for _ in 0..10 {
            println!("{}", gen_game_room_code());
        }
    }
}