/*
 * Copyright 2018-2021 EverX Labs Ltd.
 *
 * Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
 * this file except in compliance with the License.
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific EVERX DEV software governing permissions and
 * limitations under the License.
 *
 */

pub(crate) mod crypto;
pub(crate) mod debot;
pub(crate) mod handlers;
pub(crate) mod interop;
pub(crate) mod net;
pub(crate) mod processing;
#[cfg(feature = "include-zstd")]
pub(crate) mod utils;

pub(crate) mod modules;
mod registrar;
pub(crate) mod request;
pub(crate) mod runtime;

#[cfg(test)]
mod tests;
