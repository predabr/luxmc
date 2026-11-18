from pathlib import Path
import os,sys,json,re
root=Path(os.environ['LOCALAPPDATA'])/'Luxmc-build'
sys.path.insert(0,str(root/'translation-tools'))
import ctranslate2,sentencepiece

path=Path('docs/validation/ui-translation-work.json')
entries=json.loads(path.read_text(encoding='utf8'))
for entry in entries:
    if entry['key'].startswith('ui.'):
        entry.pop('en',None)
        entry.pop('es',None)
for language,directory in [('en','pt-en'),('es','en-es')]:
    package=next((root/'translation-models'/directory).glob('*/sentencepiece.model')).parent
    tokenizer=sentencepiece.SentencePieceProcessor(model_file=str(package/'sentencepiece.model'))
    translator=ctranslate2.Translator(str(package/'model'),device='cpu',compute_type='int8',inter_threads=1,intra_threads=4)
    pending=[entry for entry in entries if not entry.get(language)]
    for offset in range(0,len(pending),32):
        batch=pending[offset:offset+32]
        parts=[]
        references=[]
        for entry in batch:
            source=entry['source'] if language=='en' else entry['en']
            segments=re.split(r'({{[^}]+}}|(?<=[.!?])\s+(?=[A-ZÀ-Ý]))',source)
            references.append(segments)
            for segment in segments:
                if segment.strip() and not segment.startswith('{{'):parts.append(segment.strip())
        tokens=[tokenizer.encode(part,out_type=str) for part in parts]
        results=translator.translate_batch(tokens,beam_size=4,max_decoding_length=256,replace_unknowns=True)
        translated=iter(tokenizer.decode(result.hypotheses[0]).replace('▁',' ').strip() for result in results)
        for entry,segments in zip(batch,references):
            result=[]
            for segment in segments:
                if segment.strip() and not segment.startswith('{{'):
                    value=next(translated)
                    value=re.sub(r'\b(?:LuxMC|LuxMCs|Luxm|Lux mc)\b','Luxmc',value,flags=re.I)
                    if len(segment.strip().split())<3:value=value.rstrip('.')
                    result.append(re.match(r'^\s*',segment).group()+value+re.search(r'\s*$',segment).group())
                else:result.append(segment)
            entry[language]=''.join(result)
        path.write_text(json.dumps(entries,ensure_ascii=False,indent=2),encoding='utf8')
        print(f'{language}: {min(offset+32,len(pending))}/{len(pending)}',flush=True)
