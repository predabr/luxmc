import { readFileSync, writeFileSync, mkdirSync, copyFileSync, existsSync } from 'node:fs';
import { dirname, join } from 'node:path';
import ts from 'typescript';

const entries=JSON.parse(readFileSync('docs/validation/ui-translation-work.json','utf8'));
const overrides=JSON.parse(readFileSync('docs/validation/language-overrides.json','utf8'));
for (const entry of entries) if (overrides[entry.source]) [entry.en,entry.es]=overrides[entry.source];
if(entries.some(entry=>!entry.en||!entry.es))throw new Error('Incomplete translations');
const keys=new Map(entries.map(entry=>[entry.source,entry.key]));
for(const language of ['pt-BR','en','es']) {
    const path=`src/lib/i18n/${language}.json`;
    const dictionary=JSON.parse(readFileSync(path,'utf8'));
    dictionary.ui ||= {};
    for (const entry of entries) if(entry.key.startsWith('ui.')) dictionary.ui[entry.key.slice(3)]=language==='pt-BR'?entry.source:entry[language];
    writeFileSync(path,JSON.stringify(dictionary,null,'\t')+'\n');
}
const candidates=JSON.parse(readFileSync('docs/validation/ui-translation-candidates.json','utf8')).filter(item=> {
    if(item.type!=='literal')return true;
    if(['mundos','Qualquer Versão','vídeo','Nenhum mod de desempenho compatível','O pacote de otimização requer'].includes(item.value))return false;
    if(item.file.includes('KeybindEditorModal')&&item.value==='Todos')return false;
    const prefix=readFileSync(item.file,'utf8').slice(Math.max(0,item.start-50),item.start);
    if(/(?:\.(?:includes|startsWith|endsWith|match)\s*\(|===\s*|!==\s*)$/.test(prefix))return false;
    return true;
});
const grouped=Map.groupBy(candidates,entry=>entry.file);
let changed=0;
const backup=join(process.env.LOCALAPPDATA,'Luxmc-build','language-source-backup');
for(const [file,items] of grouped) {
    const source=readFileSync(file,'utf8');
    mkdirSync(dirname(join(backup,file)),{recursive:true});if(!existsSync(join(backup,file)))copyFileSync(file,join(backup,file));
    const render=(start,end,exclude)=>{
        let result=source.slice(start,end);
        const selected=items.filter(item=>item!==exclude&&item.start>=start&&item.end<=end).filter(item=>!items.some(parent=>parent!==item&&parent!==exclude&&parent.start>=start&&parent.end<=end&&parent.start<=item.start&&parent.end>=item.end));
        for(const item of selected.sort((a,b)=>b.start-a.start)) {
            let expression=`uiText(${JSON.stringify(keys.get(item.value))}`;
            if(item.type==='template')expression+=`, {${item.expressions.map((range,index)=>`arg${index}: (${render(range.start,range.end,item)})`).join(', ')}}`;
            expression+=')';
            let replacement=expression;
            if(item.type==='text'||item.type==='attribute') {
                const original=source.slice(item.start,item.end);
                replacement=original.match(/^\s*/)[0]+`{${expression}}`+original.match(/\s*$/)[0];
            }
            result=result.slice(0,item.start-start)+replacement+result.slice(item.end-start);
        }
        return result;
    };
    let updated=render(0,source.length);
    if(file.endsWith('.svelte')) {
        const scriptStart=updated.indexOf('<script');
        if(scriptStart>=0) {
            const bodyStart=updated.indexOf('>',scriptStart)+1;
            const bodyEnd=updated.indexOf('</script>',bodyStart);
            let script=updated.slice(bodyStart,bodyEnd);
            const ast=ts.createSourceFile(file,script,ts.ScriptTarget.Latest,true);
            const edits=[];
            for(const statement of ast.statements)if(ts.isVariableStatement(statement))for(const declaration of statement.declarationList.declarations) {
                const init=declaration.initializer;
                if(init&&/uiText\(/.test(init.getText(ast))&&(ts.isArrayLiteralExpression(init)||ts.isObjectLiteralExpression(init)||(ts.isCallExpression(init)&&init.expression.getText(ast)==='uiText')))edits.push({start:init.getStart(ast),end:init.end,text:`$derived(${init.getText(ast)})`});
            }
            for(const edit of edits.sort((a,b)=>b.start-a.start))script=script.slice(0,edit.start)+edit.text+script.slice(edit.end);
            updated=updated.slice(0,bodyStart)+(source.includes('translateUi as uiText')?'':'\nimport { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";')+script+updated.slice(bodyEnd);
        } else updated='<script lang="ts">\nimport { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";\n</script>\n'+updated;
    } else if(!source.includes('translateUi as uiText'))updated='import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";\n'+updated;
    writeFileSync(file,updated);changed++;
}
console.log(JSON.stringify({files:changed,phrases:entries.length}));
