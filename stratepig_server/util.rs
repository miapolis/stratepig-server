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

pub fn unix_now() -> u128 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

pub fn unix_now_secs() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub fn unix_timestamp_to(to: Duration) -> u128 {
    let then = SystemTime::now() + to;
    then.duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

#[macro_export]
macro_rules! unwrap_ret {
    ( $e:expr ) => {
        match $e {
            Some(x) => x,
            None => return Err(StratepigError::Unspecified),
        }
    };
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
