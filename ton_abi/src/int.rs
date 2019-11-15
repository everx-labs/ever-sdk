/*
* Copyright 2018-2019 TON DEV SOLUTIONS LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.  You may obtain a copy of the
* License at: https://ton.dev/licenses
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

use num_bigint::{BigInt, BigUint};

#[derive(Clone, Debug, PartialEq)]
pub struct Int {
    pub number: BigInt,
    pub size: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Uint {
    pub number: BigUint,
    pub size: usize,
}


impl Int {
    pub fn new(number: i128, size: usize) -> Self {
        Self { number: BigInt::from(number), size }
    }
}


impl Uint {
    pub fn new(number: u128, size: usize) -> Self {
        Self { number: BigUint::from(number), size }
    }
}
