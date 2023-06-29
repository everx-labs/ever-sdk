/*
 * Copyright 2018-2021 TON Labs LTD.
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

import {
    ApiConst,
    ApiConstValueIs,
    ApiEnumOfTypes,
    ApiField,
    ApiFunction,
    ApiFunctionInfo,
    ApiModule,
    ApiType,
    ApiTypeIs,
    Code,
    Documented,
} from "./api";

const INDENT = "    ";
const MODULES_HEADER = `
import {ResponseHandler} from "./bin";

interface IClient {
    request(
        functionName: string,
        functionParams?: any,
        responseHandler?: ResponseHandler
    ): Promise<any>;
    requestSync(
        functionName: string,
        functionParams?: any,
    ): any;
    resolve_app_request(app_request_id: number | null, result: any): Promise<void>;
    reject_app_request(app_request_id: number | null, error: any): Promise<void>;
}
`;

function isValidIdentFirstChar(c: string): boolean {
    return c >= "A" && c <= "Z" || c >= "a" && c <= "z" || c === "_";
}

function isValidIdentChar(c: string): boolean {
    return isValidIdentFirstChar(c) || (c >= "0" && c <= "9");
}

export function fixFieldName(name: string): string {
    let isValidIdent = name !== "" && isValidIdentFirstChar(name[0]);
    if (isValidIdent) {
        for (let i = 1; i < name.length; i += 1) {
            if (!isValidIdentChar(name[i])) {
                isValidIdent = false;
                break;
            }
        }
    }
    return isValidIdent ? name : `'${name.split("'").join("\\'")}'`;
}

export function typeName(fullName: string) {
    const parts = fullName.split(".");
    return parts[parts.length - 1];
}

function jsDocStart(element: Documented, indent: string = ""): string {
    return `\n${indent}/**${jsDocNext(element, indent)}`;
}

function jsDocNext(element: Documented, indent: string = ""): string {
    let ts = "";
    if (element.summary) {
        ts += jsDoc(element.summary, indent);
    }
    if (element.description) {
        ts += jsDoc("", indent);
        ts += jsDoc("@remarks", indent);
        ts += jsDoc(element.description, indent);
    }
    return ts;
}

function jsDoc(text: string, indent: string = ""): string {
    return `\n${text.split("\n").map(x => `${indent} * ${x}`).join("\n")}`;
}

function jsDocEnd(indent: string = ""): string {
    return `\n${indent} */`;
}

function elementJsDoc(element: Documented, indent: string = ""): string {
    return element.summary || element.description ?
        `${jsDocStart(element, indent)}${jsDocEnd(indent)}`
        : "";
}

function enumOfTypesJsDoc(type: ApiEnumOfTypes, indent = "") {
    let ts = jsDocStart(type);
    ts += jsDoc("\nDepends on `type` field.\n", indent);
    for (const variant of type.enum_types) {
        ts += jsDoc(`\n### \`${variant.name}\`\n`, indent);
        if (variant.summary) {
            ts += jsDoc(variant.summary, indent);
        }
    }
    ts += jsDocEnd(indent);
    return ts;
}

function getRefName(type: ApiType): string {
    return type.type === ApiTypeIs.Ref ? type.ref_name.split(".")[1] : "";
}

export class TSCode extends Code {
    language(): string {
        return "ts";
    }

    module(module: ApiModule): string {
        let ts = `// ${module.name} module\n\n`;

        for (const type of module.types) {
            if (type.type === ApiTypeIs.EnumOfTypes) {
                ts += enumOfTypesJsDoc(type);
            } else {
                ts += elementJsDoc(type);
            }

            ts += `\nexport ${this.typeDef(type, true)}`;
            if (type.type === ApiTypeIs.EnumOfTypes) {
                ts += this.typeVariantConstructors(type.name, type);
            }
        }

        const generatedAppObjects = new Set<string>();
        for (const func of module.functions) {
            const functionInfo = this.getFunctionInfo(func);
            if (functionInfo.appObject && !generatedAppObjects.has(functionInfo.appObject.name)) {
                generatedAppObjects.add(functionInfo.appObject.name);
                ts += this.appObjectInterface(functionInfo.appObject);
                ts += "\n";
                ts += this.appObjectDispatchImpl(functionInfo.appObject);
            }
        }

        ts += elementJsDoc(module);
        ts += `
export class ${Code.upperFirst(module.name)}Module {
    client: IClient;

    constructor(client: IClient) {
        this.client = client;
    }
`;

        for (const func of module.functions) {
            const info = this.getFunctionInfo(func);
            const funcDoc = () => {
                ts += jsDocStart(func, INDENT);
                if (info.params) {
                    ts += jsDoc("", INDENT);
                    ts += jsDoc(`@param {${getRefName(info.params)}} ${info.params.name}`, INDENT);
                }
                ts += jsDoc(`@returns ${getRefName(func.result)}`, INDENT);
                ts += jsDocEnd(INDENT);
            }
            funcDoc()
            ts += this.functionImpl(func);
            funcDoc()
            ts += this.syncFunctionImpl(func);
        }

        ts += "}\n\n";

        return ts;
    }

    field(field: ApiField, indent: string, includeDoc?: boolean): string {
        const name = `${fixFieldName(field.name)}${field.type === ApiTypeIs.Optional ? "?" : ""}`;
        const type = field.type === ApiTypeIs.Optional ? field.optional_inner : field;
        let ts = "";
        if (includeDoc) {
            ts += elementJsDoc(field, indent);
            ts += "\n";
        }
        ts += `${indent}${name}: ${this.type(type, indent)}`;
        return ts;
    }

    fields(fields: ApiField[], indent: string, includeDoc?: boolean): string {
        return fields.map(f => this.field(f, indent, includeDoc)).join(",\n");
    }

    typeVariant(variant: ApiField, indent: string, includeDoc?: boolean): string {
        if (variant.type === ApiTypeIs.Ref) {
            return `({\n${indent}    type: '${variant.name}'\n${indent}} & ${typeName(variant.ref_name)})`;
        } else if (variant.type === ApiTypeIs.Struct) {
            const fields = variant.struct_fields;
            let fieldsDecl: string;
            if (fields.length === 0) {
                fieldsDecl = "";
            } else {
                fieldsDecl = `\n${this.fields(fields, `${indent}    `, includeDoc)}`;
            }
            return `{\n${indent}    type: '${variant.name}'${fieldsDecl}\n${indent}}`;
        } else if (variant.type === ApiTypeIs.None) {
            return `'${variant.name}'`;
        } else {
            return this.type(variant, indent);
        }
    }

    constVariant(variant: ApiConst, indent: string, _includeDoc?: boolean): string {
        let value = "";
        switch (variant.type) {
        case ApiConstValueIs.String:
            value = `"${variant.value}"`;
            break;
        case ApiConstValueIs.None:
            value = `"${variant.name}"`;
            break;
        case ApiConstValueIs.Bool:
            value = variant.value;
            break;
        case ApiConstValueIs.Number:
            value = variant.value;
            break;
        }
        return `${indent}${variant.name} = ${value}`;

    }

    type(type: ApiType, indent: string, includeDoc?: boolean): string {
        switch (type.type) {
        case ApiTypeIs.None:
            return "void";
        case ApiTypeIs.Ref:
            if (type.ref_name === "Value" || type.ref_name === "API") {
                return "any";
            }
            return typeName(type.ref_name);
        case ApiTypeIs.Optional:
            return `${this.type(type.optional_inner, indent, includeDoc)} | null`;
        case ApiTypeIs.Struct:
            const fields = type.struct_fields;
            return `{\n${this.fields(fields, `${indent}    `, includeDoc)}\n${indent}}`;
        case ApiTypeIs.EnumOfTypes:
            return type.enum_types.map(x => this.typeVariant(x, indent, includeDoc)).join(" | ");
        case ApiTypeIs.Array:
            return `${this.type(type.array_item, indent)}[]`;
        case ApiTypeIs.EnumOfConsts:
            const variants = type.enum_consts.map(c => this.constVariant(
                c,
                `${indent}    `,
                includeDoc,
            ));
            return `{\n${variants.join(",\n")}\n${indent}}`;
        case ApiTypeIs.BigInt:
            return "bigint";
        case ApiTypeIs.Any:
            return "any";
        case ApiTypeIs.String:
            return "string";
        case ApiTypeIs.Number:
            return "number";
        case ApiTypeIs.Boolean:
            return "boolean";
        default:
            return type.type;
        }
    }

    typeDef(type: ApiField, includeDoc?: boolean): string {
        const decl = type.type === ApiTypeIs.EnumOfConsts
            ? `enum ${type.name}`
            : `type ${type.name} =`;
        return `${decl} ${this.type(type, "", includeDoc)}\n`;
    }

    paramsDecls(paramsInfo: ApiFunctionInfo): string[] {
        const decls: string[] = [];
        if (paramsInfo.params) {
            decls.push(`${paramsInfo.params.name}: ${this.type(paramsInfo.params, "")}`);
        }
        if (paramsInfo.appObject) {
            decls.push(`obj: ${paramsInfo.appObject.name}`);
        } else if (paramsInfo.hasResponseHandler) {
            decls.push("responseHandler?: ResponseHandler");
        }
        return decls;
    }

    syncParamsDecls(paramsInfo: ApiFunctionInfo): string[] {
        const decls: string[] = [];
        if (paramsInfo.params) {
            decls.push(`${paramsInfo.params.name}: ${this.type(paramsInfo.params, "")}`);
        }
        return decls;
    }

    functionInterface(func: ApiFunction): string {
        const paramsInfo = this.getFunctionInfo(func);
        const paramsDecls = this.paramsDecls(paramsInfo);
        const paramsDecl = paramsDecls.length > 0 ? `\n${paramsDecls.map(p => `    ${p},`)
            .join("\n")}\n` : "";
        const resultDecl = this.type(func.result, "");
        return `function ${func.name}(${paramsDecl}): Promise<${resultDecl}>;`;
    }

    appObjectInterface(obj: ApiModule): string {
        let ts = "";
        // for (const type of obj.types) {
        //     ts += `\ntype ${type.name} = ${type.name}Variant`;
        // }
        ts += `\nexport interface ${obj.name} {`;
        for (const f of obj.functions) {
            const isNotify = (f.result.type === ApiTypeIs.Ref) && f.result.ref_name === "";
            const paramsInfo = this.getFunctionInfo(f);
            const paramsDecls = this.paramsDecls(paramsInfo);
            const paramsDecl = paramsDecls.length > 0 ? `${paramsDecls.join(", ")}` : "";
            const resultDecl = !isNotify ? `: Promise<${this.type(f.result, "")}>` : ": void";
            ts += `\n    ${f.name}(${paramsDecl})${resultDecl},`;
        }
        ts += "\n}";
        return ts;
    }

    appObjectDispatchImpl(obj: ApiModule): string {
        let ts = `
async function dispatch${obj.name}(obj: ${obj.name}, params: ParamsOf${obj.name}, app_request_id: number | null, client: IClient) {
    try {
        let result = {};
        switch (params.type) {`;
        for (const f of obj.functions) {
            const isNotify = (f.result.type === ApiTypeIs.Ref) && f.result.ref_name === "";
            let assignment = "";
            if (!isNotify) {
                if (f.result.type !== ApiTypeIs.None) {
                    assignment = "result = await ";
                } else {
                    assignment = "await ";
                }
            }
            ts += `
            case '${TSCode.pascal(f.name.split("_"))}':
                ${assignment}obj.${f.name}(${f.params.length > 0 ? "params" : ""});
                break;`;
        }
        ts += `
        }
        client.resolve_app_request(app_request_id, { type: params.type, ...result });
    }
    catch (error) {
        client.reject_app_request(app_request_id, error);
    }
}`;
        return ts;
    }

    functionImpl(func: ApiFunction): string {
        const paramsInfo = this.getFunctionInfo(func);
        const paramsDecl = this.paramsDecls(paramsInfo).map(p => `${p}`).join(", ");
        const calls = [`'${func.module.name}.${func.name}'`];
        if (paramsInfo.params) {
            calls.push(`${paramsInfo.params.name}`);
        }
        if (paramsInfo.appObject) {
            if (!paramsInfo.params) {
                calls.push("undefined");
            }
            calls.push(`(params: any, responseType: number) => {
            if (responseType === 3) {
                dispatch${paramsInfo.appObject.name}(obj, params.request_data, params.app_request_id, this.client);
            } else if (responseType === 4) {
                dispatch${paramsInfo.appObject.name}(obj, params, null, this.client);
            }
        }`);
        } else if (paramsInfo.hasResponseHandler) {
            if (!paramsInfo.params) {
                calls.push("undefined");
            }
            calls.push("responseHandler");
        }
        return `
    ${func.name}(${paramsDecl}): Promise<${this.type(func.result, "")}> {
        return this.client.request(${calls.join(", ")});
    }\n`;
    }

    syncFunctionImpl(func: ApiFunction): string {
        const paramsInfo = this.getFunctionInfo(func);
        const paramsDecl = this.syncParamsDecls(paramsInfo).map(p => `${p}`).join(", ");
        const calls = [`'${func.module.name}.${func.name}'`];
        if (paramsInfo.params) {
            calls.push(`${paramsInfo.params.name}`);
        }
        const returnStatement = func.result.type !== ApiTypeIs.None ? "return " : "";
        return `
    ${func.name}_sync(${paramsDecl}): ${this.type(func.result, "")} {
        ${returnStatement}this.client.requestSync(${calls.join(", ")});
    }\n`;
    }

    modules(): string {
        return `
${MODULES_HEADER}
${this.api.modules.map(m => this.module(m)).join("")}
`;
    }

    private typeVariantConstructors(enumName: string, type: ApiEnumOfTypes): string {
        let ts = "";
        for (const variant of type.enum_types) {
            let params = "";
            let properties = "";
            const addFields = (fields: ApiField[]) => {
                for (const field of fields) {
                    if (params !== "") {
                        params += ", ";
                    }
                    params += `${this.field(field, "")}`;
                    properties += `        ${fixFieldName(field.name)},\n`;

                }
            };
            switch (variant.type) {
            case ApiTypeIs.Ref:
                const refType = this.findType(variant.ref_name);
                if (refType && refType.type === ApiTypeIs.Struct && refType.isInternal) {
                    addFields(refType.struct_fields);
                } else {
                    params = `params: ${typeName(variant.ref_name)}`;
                    properties = `        ...params,\n`;
                }
                break;
            case ApiTypeIs.Struct:
                addFields(variant.struct_fields);
                break;
            }
            ts +=
                `\nexport function ${TSCode.lowerFirst(enumName)}${variant.name}(${params}): ${enumName} {\n`;
            ts += `    return {\n`;
            ts += `        type: '${variant.name}',\n`;
            ts += properties;
            ts += `    };\n`;
            ts += `}\n`;
        }
        return ts;
    }

}
