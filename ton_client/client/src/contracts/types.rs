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

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct ContractABIParameter {
    pub name: String,
    #[serde(rename = "type")]
    pub parameterType: String,
}


#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct ContractABIFunction {
    pub name: String,
    pub signed: bool,
    pub inputs: Vec<ContractABIParameter>,
    pub outputs: Vec<ContractABIParameter>,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct ContractABI {
    #[serde(rename = "ABI version")]
    pub abiVersion: i32,
    pub functions: Vec<ContractABIFunction>,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct ContractPackage {
    pub abi: ContractABI,
    pub imageBase64: String,
}
