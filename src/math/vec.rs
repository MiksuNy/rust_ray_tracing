pub trait Length<T> {
    fn length(self) -> T;
}

pub trait Distance<T> {
    fn distance(a: Self, b: Self) -> T;
}

pub trait Normalized {
    fn normalized(self) -> Self;
}

pub trait Reflect {
    fn reflect(incident: Self, normal: Self) -> Self;
}

pub trait Refract {
    fn refract(incident: Self, normal: Self, eta: f32) -> Self;
}

pub trait Dot<T> {
    fn dot(a: Self, b: Self) -> T;
}

pub trait Cross<T> {
    fn cross(a: Self, b: Self) -> T;
}

pub trait Min {
    fn min(a: Self, b: Self) -> Self;
}

pub trait Max {
    fn max(a: Self, b: Self) -> Self;
}

pub trait Powf<T> {
    fn powf(a: Self, b: T) -> Self;
}

pub trait Abs {
    fn abs(self) -> Self;
}

pub trait Reversed {
    fn reversed(self) -> Self;
}

pub trait Mix<T> {
    fn mix(a: Self, b: Self, amount: T) -> Self;
}
