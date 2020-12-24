import {
    ApiConst,
    ApiEnumOfConsts,
    ApiEnumOfTypes,
    ApiField,
    ApiFunction,
    ApiModule,
    ApiStruct,
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
    return `mod_${module.name}.md`;
}

function funcRef(func: ApiFunction, module?: ApiModule): string {
    return `[${func.name}](${module ? moduleFile(module) : ""}#${func.name})`;
}

function typeRef(t: ApiField, module?: ApiModule): string {
    return `[${t.name}](${module ? moduleFile(module) : ""}#${t.name})`;
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

    tupleFields(variant: ApiStruct, indent: string): ApiField[] {
        let fields = variant.struct_fields;
        const innerType = fields[0];
        if (innerType.type === ApiTypeIs.Ref) {
            const refType = this.findType(innerType.ref_name);
            if (refType && refType.type === ApiTypeIs.Struct) {
                return this.structFields(refType, indent);
            }
        } else if (innerType.type === ApiTypeIs.Struct) {
            return this.structFields(innerType, indent);
        }
        return [
            {
                ...innerType,
                name: "value",
            },
        ];
    }

    structFields(variant: ApiStruct, _indent: string): ApiField[] {
        const fields = variant.struct_fields;
        if (fields.length === 0) {
            return fields;
        }
        return fields;
    }

    typeVariant(variant: ApiField, indent: string): string {
        let md = `When _type_ is _'${variant.name}'_\n\n`;
        md += docOf(variant);
        if (variant.type === ApiTypeIs.Struct) {
            const fields = this.structFields(variant, indent);
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
                ? `[${parts[1]}](mod_${parts[0]}.md#${parts[1]})`
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

    resolveRef(type: ApiType): ApiField | null {
        return type.type === ApiTypeIs.Ref ? this.findType(type.ref_name) : null;
    }

    functionInterface(func: ApiFunction) {
        let md = "";
        md += `## ${func.name}\n\n`;
        md += docOf(func);
        const funcInfo = this.getFunctionInfo(func);
        let code = "";
        if (funcInfo.params) {
            const params = this.resolveRef(funcInfo.params);
            if (params) {
                code += `${this.code.typeDef(params)}\n`;
            }
        }
        const result = this.resolveRef(func.result);
        if (result) {
            code += `${this.code.typeDef(result)}\n`;
        }
        code += this.code.functionInterface(func);
        md += `\`\`\`${this.code.language()}\n${code}\n\`\`\`\n`;

        if (funcInfo.params || funcInfo.hasResponseHandler) {
            md += "### Parameters\n";
            if (funcInfo.params) {
                md += this.type(funcInfo.params, "");
            }
            if (funcInfo.hasResponseHandler) {
                md += `- \`responseHandler\`?: _ResponseHandler_ – additional responses handler.`;
            }
        }
        md += "### Result\n\n";
        md += this.type(func.result, "");
        return md;
    }

    module(module: ApiModule) {
        let md = "";
        md += `# Module ${module.name}\n\n`;
        md += module.description;
        md += "\n## Functions\n";
        for (const func of module.functions) {
            md += `${funcRef(func)}${summaryOf(func)}\n\n`;
        }
        md += "## Types\n";
        for (const type of module.types) {
            md += `${typeRef(type)}${summaryOf(type)}\n\n`;
        }

        md += "\n# Functions\n";
        for (const func of module.functions) {
            md += this.functionInterface(func);
            md += "\n\n";
        }

        md += "# Types\n";
        for (const func of module.types) {
            md += this.typeDef(func);
            md += "\n\n";
        }

        return md;
    }

    modules(): string {
        let md = "";
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

    private enumOfConsts(type: ApiEnumOfConsts) {
        let md = `One of the following value:\n\n`;
        md += type.enum_consts.map(c => this.constVariant(c)).join("");
        return md;
    }
}

