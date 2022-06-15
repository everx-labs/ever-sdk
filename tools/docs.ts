import { typeName } from "./ts-code";
import { TSCode } from "./ts-code";
import {
    ApiConst,
    ApiEnumOfConsts,
    ApiEnumOfTypes,
    ApiField,
    ApiFunction,
    ApiModule,
    ApiType,
    ApiTypeIs,
    Code,
    Documented,
} from "./api";

function summaryOf(element: Documented): string {
    return element.summary ? ` – ${element.summary}` : "";
}

function descriptionOf(element: Documented): string {
    return element.description ? `<br>${element.description.split("\n").join("<br>")}\n` : "";
}

function docOf(element: Documented): string {
    let md = "";
    if (element.summary) {
        md += `${element.summary}\n\n`;
    }
    if (element.description) {
        md += `${element.description}\n\n`;
    }
    return md;
}

function moduleFile(module: ApiModule): string {
    return `mod\\_${module.name}.md`;
}

function funcRef(func: ApiFunction, module?: ApiModule): string {
    return `[${func.name}](${module ? moduleFile(module) : ""}#${func.name.toLowerCase()})`;
}

function typeRef(t: ApiField, module?: ApiModule): string {
    return `[${t.name}](${module ? moduleFile(module) : ""}#${t.name.toLowerCase()})`;
}

function appObjectTypeRef(t: ApiModule, module?: ApiModule): string {
    return `[${t.name}](${module ? moduleFile(module) : moduleFile(t)}#${t.name.toLowerCase()})`;
}

export class Docs extends Code {
    readonly code: Code;

    constructor(code: Code) {
        super(code.api);
        this.code = code;
    }

    language(): string {
        return "md";
    }

    typeDef(type: ApiField) {
        let md = "";
        md += `## ${type.name}\n`;
        md += docOf(type);
        md += `\`\`\`${this.code.language()}\n${this.code.typeDef(type)}\`\`\`\n`;
        md += this.type(type, "");
        if (type.type === ApiTypeIs.EnumOfTypes) {
            md += `\n\n${this.enumVariantConstructors(type.name, type)}`;
        }
        return md;
    }

    appObjectTypeDef(appObjectType: ApiModule) {
        let md = "";
        md += `## ${appObjectType.name}\n`;
        md += docOf(appObjectType);
        md += `\n\`\`\`${this.code.language()}\n${this.code.appObjectInterface(appObjectType)}\n\`\`\``;
        for (const func of appObjectType.functions) {
            md += "\n\n";
            md += this.functionInterface(func);
        }
        return md;
    }

    type(type: ApiType, indent: string): string {
        switch (type.type) {
        case ApiTypeIs.Ref:
            const refType = this.findType(type.ref_name);
            if (refType) {
                return this.type(refType, indent);
            }
            return `_${type.ref_name}_`;
        case ApiTypeIs.Optional:
            return `Optional value of:\n\n${this.type(type.optional_inner, indent)}`;
        case ApiTypeIs.Struct:
            return this.typeFields(type.struct_fields);
        case ApiTypeIs.EnumOfTypes:
            return this.enumOfTypes(type, indent);
        case ApiTypeIs.EnumOfConsts:
            return this.enumOfConsts(type);
        }
        return "";
    }
    typeVariant(variant: ApiField, indent: string): string {
        let md = `When _type_ is _'${variant.name}'_\n\n`;
        md += docOf(variant);
        if (variant.type === ApiTypeIs.Struct) {
            const fields = variant.struct_fields;
            let fieldsDecl: string;
            if (fields.length === 0) {
                fieldsDecl = "";
            } else {
                fieldsDecl = `\n${this.typeFields(fields)}`;
            }
            md += fieldsDecl;
        } else if (variant.type === ApiTypeIs.None) {
            md += `\`${variant.name}\``;
        } else {
            md += this.type(variant, indent);
        }
        return md;
    }

    constVariant(variant: ApiConst): string {
        let md = `- \`${this.code.constVariant(variant, "", false)}\`${summaryOf(variant)}\n`;
        md += descriptionOf(variant);
        return md;
    }

    fieldType(type: ApiType): string {
        switch (type.type) {
        case ApiTypeIs.Ref:
            if (type.ref_name === "Value") {
                return "any";
            }
            const parts = type.ref_name.split(".");
            return parts.length === 2
                ? `[${parts[1]}](mod\\_${parts[0]}.md#${parts[1].toLowerCase()})`
                : type.ref_name;
        case ApiTypeIs.Optional:
            return `${this.fieldType(type.optional_inner)}?`;
        case ApiTypeIs.Struct:
            return "struct";
        case ApiTypeIs.EnumOfTypes:
            return "enum";
        case ApiTypeIs.EnumOfConsts:
            return "const";
        case ApiTypeIs.Array:
            return `${this.fieldType(type.array_item)}[]`;
        case ApiTypeIs.String:
            return "string";
        case ApiTypeIs.Any:
            return "any";
        case ApiTypeIs.Boolean:
            return "boolean";
        case ApiTypeIs.Number:
            return "number";
        case ApiTypeIs.Generic:
            return `${type.generic_name}<${this.fieldType(type.generic_args[0])}>`;
        case ApiTypeIs.BigInt:
            return "bigint";
        case ApiTypeIs.None:
            return "void";
        default:
            return "";
        }
    }

    field(field: ApiField): string {
        const opt = field.type === ApiTypeIs.Optional ? "?" : "";
        const type = field.type === ApiTypeIs.Optional ? field.optional_inner : field;
        const name = `\`${field.name}\`${opt}: `;
        let md = `- ${name}_${this.fieldType(type)}_${summaryOf(field)}\n`;
        md += descriptionOf(field);
        return md;
    }

    resolveRef(type: ApiType, module?: ApiModule): ApiField | undefined {
        if (type.type === ApiTypeIs.Ref) {
            const resolved = this.findType(type.ref_name);
            if (resolved) {
                return resolved;
            }
            if (module) {
                for (const moduleType of module.types) {
                    if (moduleType === type || moduleType.name === type.ref_name) {
                        return moduleType;
                    }
                }
            }
        }
        return undefined;
    }

    appObjectInterface(type: ApiModule): string {
        return this.code.appObjectInterface(type);
    }

    functionInterface(func: ApiFunction) {
        let md = "";
        md += `## ${func.name}\n\n`;
        md += docOf(func);
        const funcInfo = this.getFunctionInfo(func);
        let code = "";
        const appObject = funcInfo.appObject;
        let params: ApiField | undefined = funcInfo.params;
        if (params) {
            params = this.resolveRef(params, func.module);
            if (params) {
                code += `${this.code.typeDef(params)}\n`;
            }
        }
        const result = this.resolveRef(func.result, func.module);
        if (result) {
            code += `${this.code.typeDef(result)}\n`;
        }
        code += this.code.functionInterface(func);
        md += `\`\`\`${this.code.language()}\n${code}\n\`\`\`\n`;

        if (appObject || params || funcInfo.hasResponseHandler) {
            md += "### Parameters\n";
            if (params) {
                md += this.type(params, "");
            }
            if (appObject) {
                md += `- \`obj\`: ${appObjectTypeRef(appObject)}${summaryOf(appObject)}\n\n`;
            }
            if (funcInfo.hasResponseHandler) {
                md += `- \`responseHandler\`?: _[ResponseHandler](modules.md#responsehandler)_ – additional responses handler.`;
            }
        }
        if (result) {
            md += "\n\n### Result\n\n";
            md += this.type(result, "");
        }
        return md;
    }

    module(module: ApiModule) {
        const appObjectNames = new Set<string>();
        const appObjectTypes: ApiModule[] = [];
        for (const func of module.functions) {
            const appObject = this.getFunctionInfo(func).appObject;
            if (appObject && !appObjectNames.has(appObject.name)) {
                appObjectNames.add(appObject.name);
                appObjectTypes.push(appObject);
            }
        }

        let md = "";
        md += `# Module ${module.name}\n\n`;
        md += docOf(module);
        md += "\n## Functions\n";
        for (const func of module.functions) {
            md += `${funcRef(func, module)}${summaryOf(func)}\n\n`;
        }
        md += "## Types\n";
        for (const type of module.types) {
            md += `${typeRef(type, module)}${summaryOf(type)}\n\n`;
        }
        for (const type of appObjectTypes) {
            md += `${appObjectTypeRef(type, module)}${summaryOf(type)}\n\n`;
        }

        md += "\n# Functions\n";
        for (const func of module.functions) {
            md += this.functionInterface(func);
            md += "\n\n";
        }

        md += "# Types\n";
        for (const type of module.types) {
            md += this.typeDef(type);
            md += "\n\n";
        }

        for (const type of appObjectTypes) {
            md += this.appObjectTypeDef(type);
            md += "\n\n";
        }

        return md;
    }

    readme(): string {
        let md = "";
        md += `# Types and Methods

This section contains documents describing TON SDK Types and Methods supported by various [modules](modules.md).

`;
        for (const module of this.api.modules.slice().sort(
            (left, right) => left.name.localeCompare(right.name),
        )) {
            md += `* [Module ${module.name}](${moduleFile(module)})\n`;
        }

        return md;
    }

    modules(): string {
        let md = "";
        md += `# Common Types
## ResponseHandler
\`\`\`ts
type ResponseHandler = (params: any, responseType: number) => void;
\`\`\`

Handles additional function responses.

Where:
- \`params\`: _any_ – Response parameters. Actual type depends on API function. 
- \`responseType\`: _number_ – Function specific response type.

`;
        md += "# Modules\n";
        for (const module of this.api.modules) {
            md += `## [${module.name}](${moduleFile(module)})${summaryOf(module)}\n\n`;
            for (const func of module.functions) {
                md += `${funcRef(func, module)}${summaryOf(func)}\n\n`;
            }
        }

        return md;
    }

    functionImpl(func: ApiFunction): string {
        return this.functionInterface(func);
    }

    private typeFields(fields: ApiField[]): string {
        let md = "";
        for (const field of fields) {
            md += this.field(field);
        }
        return md;
    }

    private enumOfTypes(type: ApiEnumOfTypes, indent: string) {
        let md = `Depends on value of the  \`type\` field.\n\n`;
        md += type.enum_types.map(v => this.typeVariant(v, indent)).join("\n");
        return md;
    }

    private enumVariantConstructors(enumName: string, type: ApiEnumOfTypes) {
        let md = "Variant constructors:\n\n```ts";
        for (const variant of type.enum_types) {
            let params = "";
            const addFields = (fields: ApiField[]) => {
                for (const field of fields) {
                    if (params !== "") {
                        params += ", ";
                    }
                    params += `${this.code.field(field, "")}`;
                }
            };
            switch (variant.type) {
            case ApiTypeIs.Ref:
                const refType = this.findType(variant.ref_name);
                if (refType && refType.type === ApiTypeIs.Struct && refType.isInternal) {
                    addFields(refType.struct_fields);
                } else {
                    params = `params: ${typeName(variant.ref_name)}`;
                }
                break;
            case ApiTypeIs.Struct:
                addFields(variant.struct_fields);
                break;
            }
            md += `\nfunction ${TSCode.lowerFirst(enumName)}${variant.name}(${params}): ${enumName};`;
        }
        md += "\n```";
        return md;
    }

    private enumOfConsts(type: ApiEnumOfConsts) {
        let md = `One of the following value:\n\n`;
        md += type.enum_consts.map(c => this.constVariant(c)).join("");
        return md;
    }
}

