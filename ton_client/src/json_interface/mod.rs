/*
 * Copyright 2018-2020 TON DEV SOLUTIONS LTD.
 *
 * Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
 * this file except in compliance with the License.
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific TON DEV software governing permissions and
 * limitations under the License.
 *
 */

pub(crate) mod crypto;
pub(crate) mod debot;
pub(crate) mod handlers;
pub(crate) mod interop;
pub(crate) mod net;
pub(crate) mod processing;
pub(crate) mod utils;

pub(crate) mod modules;
mod registrar;
pub(crate) mod request;
pub(crate) mod runtime;

#[cfg(test)]
mod tests;
