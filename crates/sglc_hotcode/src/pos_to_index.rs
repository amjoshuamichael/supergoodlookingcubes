use std::ops::{Mul, Div, Add, Rem};
use num::{traits::cast, NumCast};

pub fn index_to_pos<T: NumCast, U: From<(T, T, T)>>(c: usize, by: usize) -> U {
    let z = c / (by * by);
    let y = (c / by) % by;
    let x = c % by;

    unsafe {
        (
            cast::<usize, T>(x).unwrap_unchecked(), 
            cast::<usize, T>(y).unwrap_unchecked(), 
            cast::<usize, T>(z).unwrap_unchecked()
        ).into()
    }
}

pub fn pos_to_index<T: Into<(U, U, U)>, U: NumCast>(c: T, by: usize) -> usize {
    let (x, y, z) = c.into();

    unsafe {
        let x = cast::<U, usize>(x).unwrap_unchecked();
        let y = cast::<U, usize>(y).unwrap_unchecked();
        let z = cast::<U, usize>(z).unwrap_unchecked();
        
        z * by * by + y * by + x
    }
}
