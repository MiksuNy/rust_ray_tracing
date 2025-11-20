#[macro_export]
macro_rules! log_info {
    ($x:expr) => {
        eprint!("\n\x1B[38;5;14m[INFO]\x1B[0m {}", $x);
    };
    ($($x:expr),*) => {
        eprint!("\n\x1B[38;5;14m[INFO]\x1B[0m {}", format!($($x),*));
    };
}

#[macro_export]
macro_rules! log_warning {
    ($x:expr) => {
        eprint!("\n\x1B[38;5;226m[WARNING]\x1B[0m {}", $x);
    };
    ($($x:expr),*) => {
        eprint!("\n\x1B[38;5;226m[WARNING]\x1B[0m {}", format!($($x),*));
    };
}

#[macro_export]
macro_rules! log_error {
    ($x:expr) => {
        eprint!("\n\x1B[38;5;196m[ERROR]\x1B[0m {}", $x);
    };
    ($($x:expr),*) => {
        eprint!("\n\x1B[38;5;196m[ERROR]\x1B[0m {}", format!($($x),*));
    };
}
