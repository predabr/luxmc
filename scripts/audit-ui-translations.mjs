import { parse } from 'svelte/compiler';
import ts from 'typescript';
import { readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

const files = dir => readdirSync(dir, { withFileTypes: true }).flatMap(item => item.isDirectory() ? files(join(dir, item.name)) : /\.(svelte|ts)$/.test(item.name) && !/\.test\.|i18n|\.d\.ts$/.test(join(dir,item.name)) ? [join(dir,item.name)] : []);
const portuguese = value => /[áàâãéêíóôõúç]|\b(?:Adicionar|Atualizar|Baixar|Cancelar|Fechar|Salvar|Voltar|Avançar|Nenhum|Nenhuma|Criar|Excluir|Remover|Pesquisar|Buscar|Entrar|Sair|Instalar|Importar|Exportar|Solicitações|Solicitar|Amigos|Amigo|Jogar|Hospedar|Conectar|Reconectar|Carregando|Conectando|Desconectado|Todos|Nome|Recentes|Parado|Andar|Correr|Voar|Frente|Costas|Aplicar|Idioma|Geral|Contas|Comandos|Privacidade|Armazenamento|Transparência|Redefinir|Mundos|Mundo|Senha|Limpar|Copiar|Mostrar|Ocultar|Permitir|Receber|Enviar|Aceitar|Recusar|Capas|Capa|Sem capa|Ver logs|Trocar|Escolher|Selecionar|Obrigatório|Recomendado|Disponível|Indisponível|Automático|Restaurar|Detalhes|Novidades|Downloads Simultâneos|Sua|Seu|Seus|Todas|Gerando|Usar|Desativar|Ativar|Abrir|Perfil|Pasta|Memória|Padrão|Cor|Grupos|Grupo|Tela|Conta|Desligado|Ligado|Ver todas|Modificações|Arquivos|Processando|Assinar|sincronizar|sincronização|Fonte|Expulsar|Reiniciar|Sistema|Ambiente|Importando|Baixando|Servidor|Jogo|Tempo|Novo|Nova|Noturno|Falha|Erro|Total|Minutos|Segundos|Verificar|Testar|Comparar|Primeiro|Aviso|Sucesso|Aguardar|Pronto|Recursos|Desempenho|Perfis|Controles|Preparando|Bibliotecas|Aplicativo|Anexar|Solte)\b/i.test(value) || ['e','ou','para'].includes(value.trim());
const candidates=[];
const brands=new Set(['Minecraft','Luxmc','LUXMC','Lux MC','Java','Microsoft','Microsoft Azure','OAuth2','Discord','GitHub','Modrinth','CurseForge','Fabric','Quilt','Forge','NeoForge','OptiFine','OptiFine OF','Steve','Alex','Sodium','Iris','Lithium','ModernFix','FerriteCore','ImmediatelyFast','TikTok','Twitch','NameMC','Mojang','Minecon 2011','Minecon 2012','Minecon 2013','Minecon 2015','Minecon 2016','VRAM','RAM','CPU','GPU','FPS','PNG','JPG','WebP','URL','IP','LAN','UPnP','P2P','QR Code','3D','HD','PRO','GB','MB','KB','ms','MS','Linux','Windows','macOS','Vanilla','Modpacks','Mods','Shaders']);
const visible=value=>value.trim().length>1&&/[\p{L}]/u.test(value)&&!brands.has(value.trim())&&!/^(https?:|data:|[\w.-]+\.(?:com|net|br))/.test(value);
for(const title of ['Sweden','Minecraft','Stal','Pigstep','Otherside','Cat','Chirp','Relic'])brands.add(title);
for (const file of files('src')) {
    const source=readFileSync(file,'utf8');
    if (file.endsWith('.svelte')) {
        const ast=parse(source,{modern:true});
        const walk=(node,parent) => {
            if (!node || typeof node!=='object') return;
            if (node.type==='Text' && visible(node.data) && (parent?.type!=='Attribute' || ['title','alt','placeholder','aria-label','aria-description','label','description','text'].includes(parent.name))) candidates.push({file,start:node.start,end:node.end,type:parent?.type==='Attribute'?'attribute':'text',value:node.data.trim()});
            if (node.type==='Literal' && parent?.type!=='TSLiteralType' && typeof node.value==='string' && !brands.has(node.value) && (portuguese(node.value) || (parent?.type==='Property' && ['label','title','description','desc','placeholder','text'].includes(parent.key?.name))) && !/^(https?:|data:|\/|\[)/.test(node.value)) candidates.push({file,start:node.start,end:node.end,type:'literal',value:node.value});
            if (node.type==='TemplateLiteral' && node.expressions.length && node.quasis.some(q=>portuguese(q.value.cooked)) && !/^(https?:|data:|\/)/.test(node.quasis[0].value.cooked)) candidates.push({file,start:node.start,end:node.end,type:'template',value:node.quasis.map((q,i)=>q.value.cooked+(i<node.expressions.length?`{{arg${i}}}`:'')).join(''),expressions:node.expressions.map(e=>({start:e.start,end:e.end}))});
            for (const [key,value] of Object.entries(node)) if (!['parent','loc','metadata'].includes(key)) {
                if (Array.isArray(value)) value.forEach(child=>walk(child,node));
                else if (value && typeof value==='object') walk(value,node);
            }
        };
        walk(ast,null);
    } else {
        const ast=ts.createSourceFile(file,source,ts.ScriptTarget.Latest,true);
        const walk=node=>{
            if ((ts.isStringLiteral(node)||ts.isNoSubstitutionTemplateLiteral(node)) && portuguese(node.text) && !/^(https?:|data:|\/|\[)/.test(node.text)) candidates.push({file,start:node.getStart(ast),end:node.end,type:'literal',value:node.text});
            if (ts.isTemplateExpression(node) && portuguese(node.head.text+node.templateSpans.map(span=>span.literal.text).join('')) && !/^(https?:|data:|\/)/.test(node.head.text)) candidates.push({file,start:node.getStart(ast),end:node.end,type:'template',value:node.head.text+node.templateSpans.map((span,i)=>`{{arg${i}}}`+span.literal.text).join(''),expressions:node.templateSpans.map(span=>({start:span.expression.getStart(ast),end:span.expression.end}))});
            ts.forEachChild(node,walk);
        };
        walk(ast);
    }
}
const unique=[...new Set(candidates.map(item=>item.value))];
writeFileSync('docs/validation/ui-translation-candidates.json',JSON.stringify(candidates,null,2));
writeFileSync('docs/validation/ui-translation-phrases.json',JSON.stringify(unique,null,2));
console.log(JSON.stringify({occurrences:candidates.length,phrases:unique.length}));
