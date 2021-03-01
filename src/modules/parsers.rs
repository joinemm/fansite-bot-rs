use serenity::framework::standard::Args;
use std::str::FromStr;

pub fn optional_argument<T>(args: &mut Args, default_value: T) -> T
where
    T: FromStr,
{
    match args.single::<T>() {
        Ok(value) => value,
        _ => {
            args.rewind();
            default_value
        }
    }
}
