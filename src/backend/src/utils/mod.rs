mod defaults;
mod path;

pub use defaults::*;
pub use path::*;

#[macro_export]
macro_rules! custom_print {
    ($($arg:tt)*) => {
        #[cfg(not(test))]
        {
            ic_cdk::print(format!("{}", format!($($arg)*)));
        }
        #[cfg(test)]
        {
            println!("{}", format!($($arg)*));
        }
    }
}
