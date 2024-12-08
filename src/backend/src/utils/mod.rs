mod defaults;
mod path;
mod reply;

pub use defaults::*;
pub use path::*;
pub use reply::*;

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

pub(crate) fn get_current_time() -> u64 {
    #[cfg(test)]
    {
        use std::time::SystemTime;
        let duration_since_epoch = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        let timestamp_nanos = duration_since_epoch.as_nanos();
        timestamp_nanos as u64
    }
    #[cfg(not(test))]
    {
        ic_cdk::api::time()
    }
}
