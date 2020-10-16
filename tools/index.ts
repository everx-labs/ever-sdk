import os from 'os';
import fs from 'fs';
import path from 'path';
import {Code, parseApi} from './api';
import apiJson from './api.json';
import {TSCode} from './ts-code';
import {Docs} from './docs';
import program from 'commander';

function resolveDir(out: string): string {
    if (!out.startsWith('~/')) {
        return out;
    }
    return path.resolve(os.homedir(), out.substr(2));
}

function writeText(outDir: string, fileName: string, text: string) {
    console.log(fileName);
    fs.writeFileSync(path.resolve(outDir, fileName), text, 'utf8');
}

function docs(options: any) {
    const outDir = resolveDir(options.out);
    console.log('Generate docs to:', outDir);
    const code = new TSCode(parseApi(apiJson));
    const docs = new Docs(code);
    writeText(outDir, 'modules.md', docs.modules());
    for (const module of docs.api.modules) {
        writeText(outDir, `mod_${module.name}.md`, docs.module(module));
    }
}

function codeFromLanguage(language: string): Code {
    const api = parseApi(apiJson);
    switch (language) {
    case 'ts':
        return new TSCode(api);
    default:
        console.error(`error: unsupported language ${language}`);
        process.exit(1);
    }
}

function binding(options: any) {
    const language: string = options.language;
    const code = codeFromLanguage(language);
    const outDir = resolveDir(options.out);
    console.log(`Generate ${language} binding to: ${outDir}`);
    writeText(outDir, `modules.${language}`, code.modules());
}

program.command('docs')
    .option('-o, --out <dir>', 'Output directory', path.resolve(__dirname, '..', 'docs'))
    .action(docs);

program.command('binding')
    .requiredOption('-l, --language <lang>', 'Binding language')
    .requiredOption('-o, --out <dir>', 'Output directory').action(binding);

program.parse(process.argv);
