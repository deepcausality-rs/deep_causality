/*
 * Copyright (c) 2023. Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
 */

pub trait Identifiable: {
    fn id(&self) -> u64;
}
