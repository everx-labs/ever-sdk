/*
* Copyright 2018-2020 TON DEV SOLUTIONS LTD.
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

use crate::dispatch::DispatchTable;

pub(crate) mod query;

pub(crate) fn register(handlers: &mut DispatchTable) {
    handlers.spawn("queries.query", 
        |context: &mut crate::client::ClientContext, params: query::ParamsOfQuery| {
            let mut runtime = context.take_runtime()?;
            let result = runtime.block_on(query::query(context, params));
            context.runtime = Some(runtime);
            result
        });
    handlers.spawn("queries.wait.for",
        |context: &mut crate::client::ClientContext, params: query::ParamsOfWaitFor| {
            let mut runtime = context.take_runtime()?;
            let result = runtime.block_on(query::wait_for(context, params));
            context.runtime = Some(runtime);
            result
        });
    handlers.spawn("queries.subscribe",
        query::subscribe);
    handlers.spawn("queries.get.next",
        |context: &mut crate::client::ClientContext, params: query::SubscribeHandle| {
            let mut runtime = context.take_runtime()?;
            let result = runtime.block_on(query::get_next(context, params));
            context.runtime = Some(runtime);
            result
        });
    handlers.spawn("queries.unsubscribe",
        query::unsubscribe);
}
